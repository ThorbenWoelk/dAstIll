use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Sse, sse::Event},
};
use serde::Deserialize;
use std::convert::Infallible;
use std::time::{Duration, Instant};
use tokio_stream::{StreamExt, wrappers::WatchStream};

use crate::db;
use crate::models::{
    SearchMatchPayload, SearchResponsePayload, SearchStatusPayload, SearchVideoResultPayload,
};
use crate::search_query::{meaningful_search_terms, tokenize_search_terms};
use crate::security::{AccessContext, can_access_channel, can_access_video};
use crate::services::search::{
    SEARCH_RRF_K, SearchCandidate, SearchSourceKind, extract_keyword_snippet, fuse_ranked_matches,
    truncate_chunk_for_display, vector_to_json,
};
use crate::state::AppState;

use super::map_db_err;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSourceFilter {
    All,
    Transcript,
    Summary,
}

impl SearchSourceFilter {
    fn as_source_kind(self) -> Option<SearchSourceKind> {
        match self {
            Self::All => None,
            Self::Transcript => Some(SearchSourceKind::Transcript),
            Self::Summary => Some(SearchSourceKind::Summary),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Transcript => "transcript",
            Self::Summary => "summary",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub source: Option<SearchSourceFilter>,
    pub limit: Option<usize>,
    pub channel_id: Option<String>,
    pub mode: Option<SearchExecutionMode>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchExecutionMode {
    Keyword,
    Semantic,
    Hybrid,
}

impl SearchExecutionMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::Semantic => "semantic",
            Self::Hybrid => "hybrid",
        }
    }

    fn runs_keyword(self) -> bool {
        matches!(self, Self::Keyword | Self::Hybrid)
    }

    fn runs_semantic(self) -> bool {
        matches!(self, Self::Semantic | Self::Hybrid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchRetrievalMode {
    FtsOnly,
    HybridExact,
    HybridAnn,
}

impl SearchRetrievalMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::FtsOnly => "fts_only",
            Self::HybridExact => "hybrid_exact",
            Self::HybridAnn => "hybrid_ann",
        }
    }
}

#[cfg(test)]
fn resolve_search_retrieval_mode(
    embeddings_available: bool,
    vector_index_ready: bool,
) -> SearchRetrievalMode {
    if !embeddings_available {
        SearchRetrievalMode::FtsOnly
    } else if vector_index_ready {
        SearchRetrievalMode::HybridAnn
    } else {
        SearchRetrievalMode::HybridExact
    }
}

fn resolve_requested_retrieval_mode(
    execution_mode: SearchExecutionMode,
    hybrid_configured: bool,
    vector_index_ready: bool,
) -> SearchRetrievalMode {
    if execution_mode == SearchExecutionMode::Keyword || !hybrid_configured {
        SearchRetrievalMode::FtsOnly
    } else if vector_index_ready {
        SearchRetrievalMode::HybridAnn
    } else {
        SearchRetrievalMode::HybridExact
    }
}

fn resolve_semantic_retrieval_mode(
    hybrid_configured: bool,
    vector_index_ready: bool,
) -> Option<SearchRetrievalMode> {
    if !hybrid_configured {
        None
    } else if vector_index_ready {
        Some(SearchRetrievalMode::HybridAnn)
    } else {
        Some(SearchRetrievalMode::HybridExact)
    }
}

fn resolve_semantic_exact_source_kind(source: SearchSourceFilter) -> Option<SearchSourceKind> {
    match source {
        SearchSourceFilter::All => Some(SearchSourceKind::Summary),
        _ => source.as_source_kind(),
    }
}

pub async fn search(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let handler_started = Instant::now();
    let _projection_guard = state.search_projection_lock.read().await;
    let query = params.q.trim();
    if query.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Query must not be empty".to_string(),
        ));
    }

    let source = params.source.unwrap_or(SearchSourceFilter::All);
    let limit = params.limit.unwrap_or(8).clamp(1, 25);
    let execution_mode = params.mode.unwrap_or(SearchExecutionMode::Hybrid);
    if let Some(channel_id) = params.channel_id.as_deref() {
        if !can_access_channel(&access_context, channel_id) {
            return Err((StatusCode::FORBIDDEN, "Channel access denied".to_string()));
        }
    }
    let run_keyword_search = execution_mode.runs_keyword();
    let run_semantic_search = execution_mode.runs_semantic();
    let fts_terms = meaningful_search_terms(query);
    let semantic_enabled = state.search.semantic_enabled();
    let search_model = state.search.model();
    let search_status = state.search_progress.snapshot();
    let hybrid_configured = semantic_enabled && search_model.is_some();
    let semantic_retrieval_mode = if run_semantic_search {
        resolve_semantic_retrieval_mode(hybrid_configured, search_status.vector_index_ready)
    } else {
        None
    };
    let retrieval_mode = resolve_requested_retrieval_mode(
        if run_semantic_search {
            execution_mode
        } else {
            SearchExecutionMode::Keyword
        },
        hybrid_configured,
        search_status.vector_index_ready,
    );
    let fts_candidate_limit = match execution_mode {
        SearchExecutionMode::Hybrid => (limit * 8).clamp(10, 100),
        _ => (limit * 2).clamp(10, 50),
    };
    let semantic_candidate_limit = match semantic_retrieval_mode {
        Some(SearchRetrievalMode::HybridAnn) => (limit * 8).clamp(10, 100),
        Some(SearchRetrievalMode::HybridExact) => (limit * 4).clamp(10, 50),
        _ => 0,
    };
    let hyde_configured = hybrid_configured
        && state.search.hyde_model().is_some()
        && fts_terms.len() <= 4
        && run_semantic_search;

    let fts_db_started = Instant::now();
    let fts_candidates = if !run_keyword_search || fts_terms.is_empty() {
        Vec::new()
    } else {
        state
            .fts
            .search(
                query,
                source.as_source_kind(),
                params.channel_id.as_deref(),
                fts_candidate_limit,
            )
            .await
            .into_iter()
            .map(|r| {
                let mut c: SearchCandidate = r.into();
                c.chunk_text = extract_keyword_snippet(&c.chunk_text, &fts_terms);
                c
            })
            .collect()
    };
    let fts_candidates = rerank_fts_candidates(&fts_candidates, query)
        .into_iter()
        .filter(|candidate| {
            can_access_video(&access_context, &candidate.video_id, &candidate.channel_id)
        })
        .collect::<Vec<_>>();
    let fts_db_elapsed_ms = fts_db_started.elapsed().as_millis() as u64;

    let mut embedding_elapsed_ms = 0;
    let mut hybrid_db_elapsed_ms = 0;
    let mut embedding_failed = false;
    let mut hyde_triggered = false;
    let mut hyde_elapsed_ms = 0;

    let hybrid_candidates = match semantic_retrieval_mode {
        None => Vec::new(),
        Some(retrieval_mode) => {
            let Some(search_model) = search_model else {
                return Err(map_db_err("search embedding model is not configured"));
            };

            // HyDE: for short queries, synthesize a hypothetical passage and embed that
            // instead of the raw query to improve recall for dense retrieval.
            let hyde_started = Instant::now();
            let embedding_input = if hyde_configured {
                match state.search.generate_hyde_passage(query).await {
                    Ok(passage) => {
                        hyde_triggered = true;
                        passage
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "HyDE generation failed, falling back to query");
                        query.to_string()
                    }
                }
            } else {
                query.to_string()
            };
            hyde_elapsed_ms = hyde_started.elapsed().as_millis() as u64;

            let embedding_started = Instant::now();
            let embedding = match state.search.embed_texts(&[embedding_input]).await {
                Ok(embedding) => embedding,
                Err(err) => {
                    tracing::warn!(
                        error = %err,
                        execution_mode = execution_mode.as_str(),
                        retrieval_mode = retrieval_mode.as_str(),
                        "search embedding failed"
                    );
                    embedding_failed = true;
                    Vec::new()
                }
            };
            embedding_elapsed_ms = embedding_started.elapsed().as_millis() as u64;
            if embedding_failed {
                Vec::new()
            } else {
                let query_embedding_json = vector_to_json(&embedding[0]);
                let hybrid_db_started = Instant::now();
                let candidates = match retrieval_mode {
                    SearchRetrievalMode::HybridExact => {
                        db::search_exact_global_candidates(
                            &state.db,
                            &query_embedding_json,
                            search_model,
                            resolve_semantic_exact_source_kind(source),
                            params.channel_id.as_deref(),
                            semantic_candidate_limit,
                        )
                        .await
                    }
                    SearchRetrievalMode::HybridAnn => {
                        db::search_vector_candidates(
                            &state.db,
                            &query_embedding_json,
                            search_model,
                            source.as_source_kind(),
                            params.channel_id.as_deref(),
                            semantic_candidate_limit,
                        )
                        .await
                    }
                    SearchRetrievalMode::FtsOnly => Ok(Vec::new()),
                }
                .map_err(map_db_err)?;
                hybrid_db_elapsed_ms = hybrid_db_started.elapsed().as_millis() as u64;
                candidates
                    .into_iter()
                    .filter(|candidate| {
                        can_access_video(
                            &access_context,
                            &candidate.video_id,
                            &candidate.channel_id,
                        )
                    })
                    .collect()
            }
        }
    };

    if (semantic_retrieval_mode.is_none() || embedding_failed)
        && execution_mode == SearchExecutionMode::Semantic
    {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Semantic search is currently unavailable".to_string(),
        ));
    }

    let rerank_configured = state.search.rerank_model().is_some();
    let mut rerank_elapsed_ms = 0u64;

    let results = match execution_mode {
        SearchExecutionMode::Keyword => group_fts_candidates(&fts_candidates, limit),
        SearchExecutionMode::Semantic => group_ranked_candidates(&hybrid_candidates, limit),
        SearchExecutionMode::Hybrid if semantic_retrieval_mode.is_none() || embedding_failed => {
            group_fts_candidates(&fts_candidates, limit)
        }
        SearchExecutionMode::Hybrid if fts_candidates.is_empty() => {
            group_ranked_candidates(&hybrid_candidates, limit)
        }
        SearchExecutionMode::Hybrid if hybrid_candidates.is_empty() => {
            group_fts_candidates(&fts_candidates, limit)
        }
        SearchExecutionMode::Hybrid if rerank_configured => {
            // Merge both candidate lists via RRF into a single ranked flat list,
            // then let the cross-encoder reranker produce the final ordering.
            let merged = collect_rrf_candidates(&hybrid_candidates, &fts_candidates);
            let rerank_started = Instant::now();
            let reranked = match state.search.rerank_candidates(query, merged).await {
                Ok(reranked) => reranked,
                Err(err) => {
                    tracing::warn!(error = %err, "reranking failed, falling back to RRF");
                    Vec::new()
                }
            };
            rerank_elapsed_ms = rerank_started.elapsed().as_millis() as u64;
            if reranked.is_empty() {
                rank_and_group_candidates(&hybrid_candidates, &fts_candidates, limit)
            } else {
                group_ranked_candidates(&reranked, limit)
            }
        }
        SearchExecutionMode::Hybrid => {
            rank_and_group_candidates(&hybrid_candidates, &fts_candidates, limit)
        }
    };
    tracing::info!(
        query_chars = query.chars().count(),
        query_terms = query.split_whitespace().count(),
        source = source.as_str(),
        execution_mode = execution_mode.as_str(),
        retrieval_mode = retrieval_mode.as_str(),
        limit,
        fts_candidate_limit,
        semantic_candidate_limit,
        embedding_failed,
        run_keyword_search,
        run_semantic_search,
        hyde_triggered,
        hyde_elapsed_ms,
        rerank_configured,
        rerank_elapsed_ms,
        fts_candidates = fts_candidates.len(),
        hybrid_candidates = hybrid_candidates.len(),
        result_count = results.len(),
        fts_db_elapsed_ms,
        embedding_elapsed_ms,
        hybrid_db_elapsed_ms,
        elapsed_ms = handler_started.elapsed().as_millis() as u64,
        "search request completed"
    );
    Ok(Json(SearchResponsePayload {
        query: query.to_string(),
        source: source.as_str().to_string(),
        results,
    }))
}

pub async fn search_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(load_search_status_payload(&state)))
}

pub async fn search_status_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = WatchStream::new(state.search_progress.subscribe()).map(|payload| {
        let data =
            serde_json::to_string(&payload).expect("search status payload should always serialize");
        Ok(Event::default().data(data))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("ping"),
    )
}

pub(crate) fn load_search_status_payload(state: &AppState) -> SearchStatusPayload {
    state.search_progress.snapshot()
}

pub async fn rebuild_search_projection(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _projection_guard = state.search_projection_lock.write().await;
    db::reset_search_projection(&state.db)
        .await
        .map_err(map_db_err)?;
    let materials = db::list_search_progress_materials(&state.db)
        .await
        .map_err(map_db_err)?;
    state
        .search_progress
        .initialize_from_materials(
            &materials,
            state.search_progress.snapshot().available,
            false,
        )
        .await;
    Ok(StatusCode::ACCEPTED)
}

fn contains_token_phrase(text: &str, phrase_tokens: &[String]) -> bool {
    if phrase_tokens.len() < 2 {
        return false;
    }

    let text_tokens = tokenize_search_terms(text);
    text_tokens
        .windows(phrase_tokens.len())
        .any(|window| window == phrase_tokens)
}

fn count_title_term_matches(title: &str, terms: &[String]) -> usize {
    let title_terms = tokenize_search_terms(title)
        .into_iter()
        .collect::<HashSet<_>>();
    terms
        .iter()
        .filter(|term| title_terms.contains(*term))
        .count()
}


include!("frag_02.rs");
