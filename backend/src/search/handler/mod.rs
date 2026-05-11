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
use utoipa::{IntoParams, ToSchema};

use crate::db;
use crate::models::{SearchResponsePayload, SearchStatusPayload, SearchVideoResultPayload};
use crate::search::query::{meaningful_search_terms, normalize_search_text, tokenize_search_terms};
use crate::search::{SearchCandidate, SearchSourceKind, extract_keyword_snippet, vector_to_json};
use crate::security::{AccessContext, can_access_channel, can_access_video};
use crate::state::AppState;

use super::ranking::*;
use crate::handlers::map_db_err;

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchSourceFilter {
    All,
    Transcript,
    Summary,
}

impl SearchSourceFilter {
    pub(super) fn as_source_kind(self) -> Option<SearchSourceKind> {
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

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchParams {
    pub q: String,
    pub source: Option<SearchSourceFilter>,
    pub limit: Option<usize>,
    pub channel_id: Option<String>,
    pub mode: Option<SearchExecutionMode>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, ToSchema)]
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

    pub(super) fn runs_keyword(self) -> bool {
        matches!(self, Self::Keyword | Self::Hybrid)
    }

    pub(super) fn runs_semantic(self) -> bool {
        matches!(self, Self::Semantic | Self::Hybrid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SearchRetrievalMode {
    FtsOnly,
    HybridExact,
    HybridAnn,
}

impl SearchRetrievalMode {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::FtsOnly => "fts_only",
            Self::HybridExact => "hybrid_exact",
            Self::HybridAnn => "hybrid_ann",
        }
    }
}

pub(super) fn resolve_requested_retrieval_mode(
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

pub(super) fn resolve_semantic_retrieval_mode(
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

pub(super) fn resolve_semantic_exact_source_kind(
    source: SearchSourceFilter,
) -> Option<SearchSourceKind> {
    match source {
        SearchSourceFilter::All => Some(SearchSourceKind::Summary),
        _ => source.as_source_kind(),
    }
}

#[utoipa::path(
    get,
    path = "/api/search",
    params(SearchParams),
    responses(
        (status = 200, description = "Search results", body = SearchResponsePayload),
        (status = 400, description = "Invalid query", body = String),
        (status = 403, description = "Channel access denied", body = String),
        (status = 503, description = "Semantic search unavailable", body = String)
    )
)]
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
    let retrieval_query = normalize_search_text(query);
    if let Some(channel_id) = params.channel_id.as_deref() {
        if !can_access_channel(&access_context, channel_id) {
            return Err((StatusCode::FORBIDDEN, "Channel access denied".to_string()));
        }
    }
    let run_keyword_search =
        execution_mode.runs_keyword() || execution_mode == SearchExecutionMode::Semantic;
    let run_semantic_search = execution_mode.runs_semantic();
    let fts_terms = meaningful_search_terms(&retrieval_query);
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
    let fts_candidates = rerank_fts_candidates(&fts_candidates, &retrieval_query)
        .into_iter()
        .filter(|candidate| {
            can_access_video(&access_context, &candidate.video_id, &candidate.channel_id)
        })
        .collect::<Vec<_>>();
    let semantic_keyword_rescue_candidates =
        exact_keyword_rescue_candidates(&retrieval_query, &fts_candidates);
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
                match state.search.generate_hyde_passage(&retrieval_query).await {
                    Ok(passage) => {
                        hyde_triggered = true;
                        passage
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "HyDE generation failed, falling back to query");
                        retrieval_query.clone()
                    }
                }
            } else {
                retrieval_query.clone()
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

    let semantic_keyword_rescue_active = execution_mode == SearchExecutionMode::Semantic
        && should_rescue_semantic_results(
            &retrieval_query,
            &semantic_keyword_rescue_candidates,
            &hybrid_candidates,
        );

    let results = match execution_mode {
        SearchExecutionMode::Keyword => group_fts_candidates(&fts_candidates, limit),
        SearchExecutionMode::Semantic if semantic_keyword_rescue_active => {
            group_fts_candidates(&semantic_keyword_rescue_candidates, limit)
        }
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
            let reranked = match state
                .search
                .rerank_candidates(&retrieval_query, merged)
                .await
            {
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
    let hybrid_keyword_rescue_active = execution_mode == SearchExecutionMode::Hybrid
        && should_promote_hybrid_keyword_rescue(&semantic_keyword_rescue_candidates, &results);
    let results = if hybrid_keyword_rescue_active {
        promote_hybrid_keyword_rescue_results(results, &semantic_keyword_rescue_candidates)
    } else {
        results
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
        semantic_keyword_rescue_candidates = semantic_keyword_rescue_candidates.len(),
        semantic_keyword_rescue_active,
        hybrid_keyword_rescue_active,
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

#[utoipa::path(
    get,
    path = "/api/search/status",
    responses(
        (status = 200, description = "Search status payload", body = SearchStatusPayload)
    )
)]
pub async fn search_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(load_search_status_payload(&state)))
}

#[utoipa::path(
    get,
    path = "/api/search/status/stream",
    responses(
        (status = 200, description = "Server-sent search status stream", body = String, content_type = "text/event-stream")
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/search/rebuild",
    responses(
        (status = 202, description = "Search rebuild accepted"),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn rebuild_search_projection(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _projection_guard = state.search_projection_lock.write().await;
    state.fts.clear().await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to clear keyword search index: {err}"),
        )
    })?;
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

fn candidate_combined_terms(candidate: &SearchCandidate) -> HashSet<String> {
    let mut terms = tokenize_search_terms(&candidate.video_title)
        .into_iter()
        .collect::<HashSet<_>>();

    if let Some(section_title) = candidate.section_title.as_deref() {
        terms.extend(tokenize_search_terms(section_title));
    }
    terms.extend(tokenize_search_terms(&candidate.chunk_text));
    terms
}

fn candidate_has_exact_query_signal(candidate: &SearchCandidate, query: &str) -> bool {
    let meaningful_terms = meaningful_search_terms(query);
    if meaningful_terms.len() < 2 {
        return false;
    }

    let raw_phrase_tokens = tokenize_search_terms(query);
    let exact_phrase_match = contains_token_phrase(&candidate.video_title, &raw_phrase_tokens)
        || contains_token_phrase(&candidate.chunk_text, &raw_phrase_tokens)
        || candidate
            .section_title
            .as_deref()
            .is_some_and(|title| contains_token_phrase(title, &raw_phrase_tokens))
        || contains_token_phrase(&candidate.video_title, &meaningful_terms)
        || contains_token_phrase(&candidate.chunk_text, &meaningful_terms)
        || candidate
            .section_title
            .as_deref()
            .is_some_and(|title| contains_token_phrase(title, &meaningful_terms));

    let combined_terms = candidate_combined_terms(candidate);
    let combined_contains_all_terms = meaningful_terms
        .iter()
        .all(|term| combined_terms.contains(term));

    exact_phrase_match
        || combined_contains_all_terms
        || count_title_term_matches(&candidate.video_title, &meaningful_terms)
            == meaningful_terms.len()
}

fn exact_keyword_rescue_candidates(
    query: &str,
    candidates: &[SearchCandidate],
) -> Vec<SearchCandidate> {
    candidates
        .iter()
        .filter(|candidate| candidate_has_exact_query_signal(candidate, query))
        .cloned()
        .collect()
}

fn should_rescue_semantic_results(
    query: &str,
    keyword_rescue_candidates: &[SearchCandidate],
    semantic_candidates: &[SearchCandidate],
) -> bool {
    if keyword_rescue_candidates.is_empty() {
        return false;
    }

    if semantic_candidates.is_empty() {
        return true;
    }

    !semantic_candidates
        .iter()
        .take(3)
        .any(|candidate| candidate_has_exact_query_signal(candidate, query))
}

fn keyword_rescue_video_ids(candidates: &[SearchCandidate]) -> Vec<String> {
    let mut seen = HashSet::new();
    candidates
        .iter()
        .filter_map(|candidate| {
            if seen.insert(candidate.video_id.clone()) {
                Some(candidate.video_id.clone())
            } else {
                None
            }
        })
        .collect()
}

fn should_promote_hybrid_keyword_rescue(
    keyword_rescue_candidates: &[SearchCandidate],
    results: &[SearchVideoResultPayload],
) -> bool {
    let rescue_video_ids = keyword_rescue_video_ids(keyword_rescue_candidates);
    if rescue_video_ids.is_empty() || results.is_empty() {
        return false;
    }

    !rescue_video_ids
        .iter()
        .any(|video_id| video_id == &results[0].video_id)
}

fn promote_hybrid_keyword_rescue_results(
    results: Vec<SearchVideoResultPayload>,
    keyword_rescue_candidates: &[SearchCandidate],
) -> Vec<SearchVideoResultPayload> {
    let rescue_video_ids = keyword_rescue_video_ids(keyword_rescue_candidates);
    if rescue_video_ids.is_empty() {
        return results;
    }

    let rescue_order = rescue_video_ids
        .into_iter()
        .enumerate()
        .map(|(index, video_id)| (video_id, index))
        .collect::<HashMap<_, _>>();

    let mut rescue_results = Vec::new();
    let mut remaining_results = Vec::new();

    for result in results {
        if let Some(position) = rescue_order.get(&result.video_id).copied() {
            rescue_results.push((position, result));
        } else {
            remaining_results.push(result);
        }
    }

    rescue_results.sort_by_key(|(position, _)| *position);
    rescue_results
        .into_iter()
        .map(|(_, result)| result)
        .chain(remaining_results)
        .collect()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
