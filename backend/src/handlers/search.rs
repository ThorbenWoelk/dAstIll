use std::cmp::Ordering;
use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::db;
use crate::models::{
    SearchMatchPayload, SearchResponsePayload, SearchStatusPayload, SearchVideoResultPayload,
};
use crate::services::search::{
    SEARCH_RRF_K, SearchCandidate, SearchSourceKind, fuse_ranked_matches,
    truncate_chunk_for_display, vector_to_json,
};
use crate::state::AppState;

use super::{map_db_err, map_internal_err};

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

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
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
    let candidate_limit = (limit * 8).clamp(10, 100);
    let fts_query = build_fts_query(query);
    let conn = state.db.connect();
    let semantic_enabled = state.search.semantic_enabled();
    let search_model = state.search.model();
    let vector_index_ready = if semantic_enabled {
        db::has_vector_index(&conn).await.map_err(map_db_err)?
    } else {
        false
    };
    let embeddings_available = state.search.is_available().await;
    let retrieval_mode = resolve_search_retrieval_mode(embeddings_available, vector_index_ready);

    let fts_candidates = if fts_query.is_empty() {
        Vec::new()
    } else {
        db::search_fts_candidates(
            &conn,
            &fts_query,
            if semantic_enabled { search_model } else { None },
            source.as_source_kind(),
            params.channel_id.as_deref(),
            candidate_limit,
        )
        .await
        .map_err(map_db_err)?
    };

    let hybrid_candidates = match retrieval_mode {
        SearchRetrievalMode::FtsOnly => Vec::new(),
        SearchRetrievalMode::HybridExact => {
            if fts_candidates.is_empty() {
                Vec::new()
            } else {
                let Some(search_model) = search_model else {
                    return Err(map_internal_err("search embedding model is not configured"));
                };
                let embedding = state
                    .search
                    .embed_texts(&[query.to_string()])
                    .await
                    .map_err(map_internal_err)?;
                let query_embedding_json = vector_to_json(&embedding[0]);
                let candidate_ids = fts_candidates
                    .iter()
                    .filter_map(|candidate| candidate.chunk_id.parse::<i64>().ok())
                    .collect::<Vec<_>>();
                db::search_exact_candidates(
                    &conn,
                    &query_embedding_json,
                    search_model,
                    &candidate_ids,
                    candidate_limit,
                )
                .await
                .map_err(map_db_err)?
            }
        }
        SearchRetrievalMode::HybridAnn => {
            let Some(search_model) = search_model else {
                return Err(map_internal_err("search embedding model is not configured"));
            };
            let embedding = state
                .search
                .embed_texts(&[query.to_string()])
                .await
                .map_err(map_internal_err)?;
            let query_embedding_json = vector_to_json(&embedding[0]);
            db::search_vector_candidates(
                &conn,
                &query_embedding_json,
                search_model,
                source.as_source_kind(),
                params.channel_id.as_deref(),
                candidate_limit,
            )
            .await
            .map_err(map_db_err)?
        }
    };

    let results = rank_and_group_candidates(&hybrid_candidates, &fts_candidates, limit);
    Ok(Json(SearchResponsePayload {
        query: query.to_string(),
        source: source.as_str().to_string(),
        results,
    }))
}

pub async fn search_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.connect();
    Ok(Json(load_search_status_payload(&state, &conn).await?))
}

pub(crate) async fn load_search_status_payload(
    state: &AppState,
    conn: &libsql::Connection,
) -> Result<SearchStatusPayload, (StatusCode, String)> {
    let _projection_guard = state.search_projection_lock.read().await;
    let counts = db::get_search_source_counts(conn)
        .await
        .map_err(map_db_err)?;
    let available = state.search.is_available().await;
    let vector_index_ready = if state.search.semantic_enabled() {
        db::has_vector_index(conn).await.map_err(map_db_err)?
    } else {
        false
    };
    let retrieval_mode = resolve_search_retrieval_mode(available, vector_index_ready);
    Ok(SearchStatusPayload {
        available,
        model: state.search.model().unwrap_or_default().to_string(),
        dimensions: if state.search.semantic_enabled() {
            state.search.dimensions()
        } else {
            0
        },
        pending: counts.pending,
        indexing: counts.indexing,
        ready: counts.ready,
        failed: counts.failed,
        total_sources: counts.total_sources,
        vector_index_ready,
        retrieval_mode: retrieval_mode.as_str().to_string(),
    })
}

pub async fn rebuild_search_projection(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _projection_guard = state.search_projection_lock.write().await;
    let conn = state.db.connect();
    db::reset_search_projection(&conn)
        .await
        .map_err(map_db_err)?;
    Ok(StatusCode::ACCEPTED)
}

fn build_fts_query(query: &str) -> String {
    query
        .split(|character: char| {
            !(character.is_alphanumeric() || matches!(character, '_' | '-' | '.'))
        })
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
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
        SearchSourceFilter, build_fts_query, rank_and_group_candidates,
        resolve_search_retrieval_mode,
    };
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
        }
    }

    #[test]
    fn build_fts_query_quotes_search_terms() {
        assert_eq!(
            build_fts_query("semantic search qwen3-embedding"),
            "\"semantic\" OR \"search\" OR \"qwen3-embedding\""
        );
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
}
