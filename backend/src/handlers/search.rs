use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use axum::{
    Json,
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
    let fts_candidates = rerank_fts_candidates(&fts_candidates, query);
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

fn rerank_fts_candidates(candidates: &[SearchCandidate], query: &str) -> Vec<SearchCandidate> {
    let meaningful_terms = meaningful_search_terms(query);
    if candidates.len() <= 1 || meaningful_terms.is_empty() {
        return candidates.to_vec();
    }

    let raw_phrase_tokens = tokenize_search_terms(query);
    let meaningful_phrase_tokens = if meaningful_terms.len() >= 2 {
        Some(meaningful_terms.clone())
    } else {
        None
    };
    let mut ranked = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let exact_phrase_match =
                contains_token_phrase(&candidate.video_title, &raw_phrase_tokens)
                    || contains_token_phrase(&candidate.chunk_text, &raw_phrase_tokens)
                    || candidate
                        .section_title
                        .as_deref()
                        .is_some_and(|title| contains_token_phrase(title, &raw_phrase_tokens))
                    || meaningful_phrase_tokens
                        .as_ref()
                        .is_some_and(|phrase_tokens| {
                            contains_token_phrase(&candidate.video_title, phrase_tokens)
                                || contains_token_phrase(&candidate.chunk_text, phrase_tokens)
                                || candidate.section_title.as_deref().is_some_and(|title| {
                                    contains_token_phrase(title, phrase_tokens)
                                })
                        });
            let title_term_matches =
                count_title_term_matches(&candidate.video_title, &meaningful_terms);
            let title_contains_all_terms = title_term_matches == meaningful_terms.len();
            (
                exact_phrase_match,
                candidate.source_kind == SearchSourceKind::Summary,
                title_contains_all_terms,
                title_term_matches,
                index,
                candidate.clone(),
            )
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| right.2.cmp(&left.2))
            .then_with(|| right.3.cmp(&left.3))
            .then_with(|| left.4.cmp(&right.4))
    });

    ranked
        .into_iter()
        .map(|(_, _, _, _, _, candidate)| candidate)
        .collect()
}

fn group_ranked_candidates(
    candidates: &[SearchCandidate],
    limit: usize,
) -> Vec<SearchVideoResultPayload> {
    let mut grouped = HashMap::<String, SearchVideoResultPayload>::new();
    let mut best_ranks = HashMap::<String, usize>::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let rank = index + 1;
        let group = grouped
            .entry(candidate.video_id.clone())
            .or_insert_with(|| SearchVideoResultPayload {
                video_id: candidate.video_id.clone(),
                channel_id: candidate.channel_id.clone(),
                channel_name: candidate.channel_name.clone(),
                video_title: candidate.video_title.clone(),
                published_at: candidate.published_at.clone(),
                matches: Vec::new(),
            });

        let existing = group
            .matches
            .iter()
            .position(|existing| existing.source == candidate.source_kind);
        let score = 1.0 / (SEARCH_RRF_K + rank as f32);
        let payload = SearchMatchPayload {
            source: candidate.source_kind,
            section_title: candidate.section_title.clone(),
            snippet: truncate_chunk_for_display(&candidate.chunk_text),
            score,
            start_sec: candidate.start_sec,
        };

        match existing {
            Some(index) if payload.score > group.matches[index].score => {
                group.matches[index] = payload;
            }
            None => group.matches.push(payload),
            _ => {}
        }

        best_ranks
            .entry(candidate.video_id.clone())
            .and_modify(|best| *best = (*best).min(rank))
            .or_insert(rank);
    }

    let mut results = grouped.into_values().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        let left_rank = best_ranks
            .get(&left.video_id)
            .copied()
            .unwrap_or(usize::MAX);
        let right_rank = best_ranks
            .get(&right.video_id)
            .copied()
            .unwrap_or(usize::MAX);
        left_rank
            .cmp(&right_rank)
            .then_with(|| right.published_at.cmp(&left.published_at))
    });
    for result in &mut results {
        result.matches.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
        });
    }
    results.truncate(limit);
    results
}

fn group_fts_candidates(
    candidates: &[SearchCandidate],
    limit: usize,
) -> Vec<SearchVideoResultPayload> {
    group_ranked_candidates(candidates, limit)
}

/// Merge vector and FTS candidate lists via RRF, returning a flat deduplicated list
/// ordered by descending fused score. Used as input to the cross-encoder reranker.
fn collect_rrf_candidates(
    vector_candidates: &[SearchCandidate],
    fts_candidates: &[SearchCandidate],
) -> Vec<SearchCandidate> {
    let vector_ranks: Vec<(&str, usize)> = vector_candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (c.chunk_id.as_str(), i + 1))
        .collect();
    let fts_ranks: Vec<(&str, usize)> = fts_candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (c.chunk_id.as_str(), i + 1))
        .collect();
    let fused = fuse_ranked_matches(&vector_ranks, &fts_ranks, SEARCH_RRF_K);

    let mut by_id: std::collections::HashMap<&str, &SearchCandidate> =
        std::collections::HashMap::new();
    for c in vector_candidates.iter().chain(fts_candidates.iter()) {
        by_id.insert(c.chunk_id.as_str(), c);
    }

    fused
        .into_iter()
        .filter_map(|(chunk_id, _score)| by_id.get(chunk_id.as_str()).copied().cloned())
        .collect()
}

fn rank_and_group_candidates(
    vector_candidates: &[SearchCandidate],
    fts_candidates: &[SearchCandidate],
    limit: usize,
) -> Vec<SearchVideoResultPayload> {
    let vector_ranks = vector_candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate.chunk_id.as_str(), index + 1))
        .collect::<Vec<_>>();
    let fts_ranks = fts_candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate.chunk_id.as_str(), index + 1))
        .collect::<Vec<_>>();
    let fused = fuse_ranked_matches(&vector_ranks, &fts_ranks, SEARCH_RRF_K);

    let mut candidates = HashMap::<String, SearchCandidate>::new();
    for candidate in vector_candidates.iter().chain(fts_candidates.iter()) {
        candidates.insert(candidate.chunk_id.clone(), candidate.clone());
    }

    let mut grouped = HashMap::<String, SearchVideoResultPayload>::new();
    let mut best_scores = HashMap::<String, f32>::new();

    for (chunk_id, score) in fused {
        let Some(candidate) = candidates.get(&chunk_id) else {
            continue;
        };

        let group = grouped
            .entry(candidate.video_id.clone())
            .or_insert_with(|| SearchVideoResultPayload {
                video_id: candidate.video_id.clone(),
                channel_id: candidate.channel_id.clone(),
                channel_name: candidate.channel_name.clone(),
                video_title: candidate.video_title.clone(),
                published_at: candidate.published_at.clone(),
                matches: Vec::new(),
            });

        let existing = group
            .matches
            .iter()
            .position(|existing| existing.source == candidate.source_kind);
        let payload = SearchMatchPayload {
            source: candidate.source_kind,
            section_title: candidate.section_title.clone(),
            snippet: truncate_chunk_for_display(&candidate.chunk_text),
            score,
            start_sec: candidate.start_sec,
        };

        match existing {
            Some(index) if payload.score > group.matches[index].score => {
                group.matches[index] = payload;
            }
            None => group.matches.push(payload),
            _ => {}
        }

        best_scores
            .entry(candidate.video_id.clone())
            .and_modify(|best| *best = best.max(score))
            .or_insert(score);
    }

    let mut results = grouped.into_values().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        let right_score = best_scores
            .get(&right.video_id)
            .copied()
            .unwrap_or_default();
        let left_score = best_scores.get(&left.video_id).copied().unwrap_or_default();
        right_score
            .partial_cmp(&left_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.published_at.cmp(&left.published_at))
    });
    for result in &mut results {
        result.matches.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
        });
    }
    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use super::{
        SearchExecutionMode, SearchRetrievalMode, SearchSourceFilter, group_fts_candidates,
        group_ranked_candidates, rank_and_group_candidates, rerank_fts_candidates,
        resolve_requested_retrieval_mode, resolve_search_retrieval_mode,
        resolve_semantic_exact_source_kind, resolve_semantic_retrieval_mode,
    };
    use crate::search_query::build_fts_query;
    use crate::services::search::{SearchCandidate, SearchSourceKind};

    fn candidate(chunk_id: &str, video_id: &str, source_kind: SearchSourceKind) -> SearchCandidate {
        SearchCandidate {
            chunk_id: chunk_id.to_string(),
            video_id: video_id.to_string(),
            channel_id: "channel".to_string(),
            channel_name: "Channel".to_string(),
            video_title: "Title".to_string(),
            source_kind,
            section_title: None,
            chunk_text: "A detailed snippet about semantic search.".to_string(),
            published_at: "2026-03-12T00:00:00Z".to_string(),
            start_sec: None,
        }
    }

    #[test]
    fn build_fts_query_quotes_search_terms() {
        assert_eq!(
            build_fts_query("semantic search qwen3-embedding"),
            "\"semantic\" AND \"search\" AND \"qwen3-embedding\""
        );
    }

    #[test]
    fn build_fts_query_drops_broad_question_stopwords_but_keeps_technical_terms() {
        assert_eq!(
            build_fts_query("what is the best db in town"),
            "\"db\" AND \"town\""
        );
        assert_eq!(build_fts_query("how to use ai"), "\"use\" AND \"ai\"");
    }

    #[test]
    fn build_fts_query_deduplicates_and_caps_terms() {
        assert_eq!(
            build_fts_query("rust rust tokio axum libsql semantic search"),
            "\"rust\" AND \"tokio\" AND \"axum\" AND \"libsql\""
        );
    }

    #[test]
    fn rerank_fts_candidates_prioritizes_phrase_then_summary_then_title() {
        let results = rerank_fts_candidates(
            &[
                SearchCandidate {
                    video_title: "town database guide".to_string(),
                    source_kind: SearchSourceKind::Transcript,
                    chunk_text: "db town".to_string(),
                    ..candidate("a", "video-1", SearchSourceKind::Transcript)
                },
                SearchCandidate {
                    video_title: "DB choices".to_string(),
                    source_kind: SearchSourceKind::Summary,
                    chunk_text: "database options across the town with db comparisons".to_string(),
                    ..candidate("b", "video-2", SearchSourceKind::Summary)
                },
                SearchCandidate {
                    video_title: "Other video".to_string(),
                    source_kind: SearchSourceKind::Transcript,
                    chunk_text: "a db for every town".to_string(),
                    ..candidate("c", "video-3", SearchSourceKind::Transcript)
                },
            ],
            "db town",
        );

        assert_eq!(results[0].video_id, "video-1");
        assert_eq!(results[1].video_id, "video-2");
        assert_eq!(results[2].video_id, "video-3");
    }

    #[test]
    fn grouping_keeps_best_match_per_source_kind() {
        let results = rank_and_group_candidates(
            &[
                candidate("a", "video-1", SearchSourceKind::Summary),
                candidate("b", "video-1", SearchSourceKind::Transcript),
            ],
            &[candidate("b", "video-1", SearchSourceKind::Transcript)],
            10,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 2);
    }

    #[test]
    fn source_filter_all_maps_to_no_db_filter() {
        assert_eq!(SearchSourceFilter::All.as_source_kind(), None);
        assert_eq!(
            SearchSourceFilter::Summary.as_source_kind(),
            Some(SearchSourceKind::Summary)
        );
    }

    #[test]
    fn semantic_exact_fallback_prefers_summaries_for_all_sources() {
        assert_eq!(
            resolve_semantic_exact_source_kind(SearchSourceFilter::All),
            Some(SearchSourceKind::Summary)
        );
        assert_eq!(
            resolve_semantic_exact_source_kind(SearchSourceFilter::Transcript),
            Some(SearchSourceKind::Transcript)
        );
    }

    #[test]
    fn fts_grouping_preserves_bm25_rank_order() {
        let results = group_fts_candidates(
            &[
                candidate("a", "video-1", SearchSourceKind::Summary),
                candidate("b", "video-2", SearchSourceKind::Transcript),
                candidate("c", "video-1", SearchSourceKind::Transcript),
            ],
            10,
        );

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].video_id, "video-1");
        assert_eq!(results[0].matches.len(), 2);
        assert_eq!(results[1].video_id, "video-2");
    }

    #[test]
    fn fts_grouping_respects_limit() {
        let results = group_fts_candidates(
            &[
                candidate("a", "video-1", SearchSourceKind::Summary),
                candidate("b", "video-2", SearchSourceKind::Transcript),
            ],
            1,
        );

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].video_id, "video-1");
    }

    #[test]
    fn semantic_grouping_preserves_rank_order() {
        let results = group_ranked_candidates(
            &[
                candidate("a", "video-2", SearchSourceKind::Summary),
                candidate("b", "video-1", SearchSourceKind::Transcript),
            ],
            10,
        );

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].video_id, "video-2");
    }

    #[test]
    fn retrieval_mode_falls_back_to_fts_without_vector_index() {
        assert_eq!(
            resolve_search_retrieval_mode(false, false).as_str(),
            "fts_only"
        );
        assert_eq!(
            resolve_search_retrieval_mode(true, false).as_str(),
            "hybrid_exact"
        );
        assert_eq!(
            resolve_search_retrieval_mode(true, true).as_str(),
            "hybrid_ann"
        );
    }

    #[test]
    fn semantic_retrieval_mode_is_disabled_when_semantic_search_is_not_configured() {
        assert_eq!(resolve_semantic_retrieval_mode(false, false), None);
        assert_eq!(
            resolve_semantic_retrieval_mode(true, false),
            Some(SearchRetrievalMode::HybridExact)
        );
        assert_eq!(
            resolve_semantic_retrieval_mode(true, true),
            Some(SearchRetrievalMode::HybridAnn)
        );
    }

    #[test]
    fn keyword_mode_forces_fts_only_even_when_hybrid_is_ready() {
        assert!(SearchExecutionMode::Keyword.runs_keyword());
        assert!(!SearchExecutionMode::Keyword.runs_semantic());
        assert!(!SearchExecutionMode::Semantic.runs_keyword());
        assert!(SearchExecutionMode::Semantic.runs_semantic());
        assert!(SearchExecutionMode::Hybrid.runs_keyword());
        assert!(SearchExecutionMode::Hybrid.runs_semantic());
        assert_eq!(
            resolve_requested_retrieval_mode(SearchExecutionMode::Keyword, true, true),
            SearchRetrievalMode::FtsOnly,
        );
        assert_eq!(
            resolve_requested_retrieval_mode(SearchExecutionMode::Semantic, true, true),
            SearchRetrievalMode::HybridAnn,
        );
        assert_eq!(
            resolve_requested_retrieval_mode(SearchExecutionMode::Hybrid, true, false),
            SearchRetrievalMode::HybridExact,
        );
    }
}
