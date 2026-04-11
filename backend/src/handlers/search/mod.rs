mod ranking;

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
use utoipa::{IntoParams, ToSchema};

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
use ranking::*;

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
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

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use axum::{
        Extension,
        body::to_bytes,
        extract::{Query, State},
        response::IntoResponse,
    };
    use reqwest::Client;
    use tokio::sync::RwLock;

    use super::{SearchExecutionMode, SearchParams, search};
    use crate::{
        models::SearchResponsePayload,
        search_progress::SearchProgress,
        security::{AccessContext, AccessRole, AuthState},
        services::fts::FtsSourceMeta,
        services::{
            ChatService, CloudCooldown, FtsChunk, InputGuardrailService, OllamaCore,
            OpenAlexPlannerService, OpenAlexService, PodcastFeedService, SearchService,
            SearchSourceKind, SummarizerService, SummaryEvaluatorService, TranscriptCooldown,
            TranscriptService, UserActivity, WebsiteService, YouTubeQuotaCooldown, YouTubeService,
        },
        state::AppState,
    };

    async fn test_app_state() -> AppState {
        let db = crate::db::Store::for_test().await;
        let cooldown = Arc::new(CloudCooldown::cloud());
        let security =
            Arc::new(crate::config::SecurityRuntimeConfig::from_env().expect("security config"));
        AppState {
            db,
            read_cache: Arc::new(crate::read_cache::ReadCache::default()),
            security: security.clone(),
            request_rate_limiter: crate::security::rate_limiter(security.as_ref()),
            search_auto_create_vector_index: false,
            search_projection_lock: Arc::new(RwLock::new(())),
            search_progress: Arc::new(SearchProgress::new(
                None,
                crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
                false,
            )),
            youtube: Arc::new(YouTubeService::with_client(Client::new())),
            openalex_planner: Arc::new(OpenAlexPlannerService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            openalex: Arc::new(OpenAlexService::with_client(Client::new())),
            podcast_feed: Arc::new(PodcastFeedService::with_client(Client::new())),
            website: Arc::new(WebsiteService::with_client(Client::new())),
            transcript: Arc::new(TranscriptService::with_path("/usr/bin/false")),
            tts: None,
            summarizer: Arc::new(SummarizerService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            summary_evaluator: Arc::new(SummaryEvaluatorService::new(
                OllamaCore::new("://invalid-url", "qwen3.5:397b-cloud")
                    .with_cloud_cooldown(cooldown.clone()),
            )),
            search: Arc::new(SearchService::with_config(
                "://invalid-url",
                None,
                crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
                false,
            )),
            chat: Arc::new(ChatService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            input_guardrails: Arc::new(InputGuardrailService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
                Vec::new(),
                Vec::new(),
            )),
            analytics: None,
            active_replies: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            conversation_store_lock: Arc::new(tokio::sync::Mutex::new(())),
            fts: Arc::new(crate::services::FtsIndex::new().await.expect("fts index")),
            anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
            mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            cloud_cooldown: cooldown,
            youtube_quota_cooldown: Arc::new(YouTubeQuotaCooldown::youtube_quota()),
            transcript_cooldown: Arc::new(TranscriptCooldown::transcript()),
            user_activity: Arc::new(UserActivity::from_env()),
        }
    }

    #[tokio::test]
    async fn keyword_search_returns_only_accessible_results() {
        let state = test_app_state().await;

        for (video_id, channel_id, title) in [
            ("video-channel", "channel-a", "Scoped channel result"),
            (
                "video-other",
                "channel-hidden",
                "Explicit video membership result",
            ),
            ("video-forbidden", "channel-hidden", "Forbidden result"),
        ] {
            state
                .fts
                .upsert_source(
                    FtsSourceMeta {
                        video_id,
                        source_kind: SearchSourceKind::Transcript,
                        channel_id,
                        channel_name: "Channel",
                        video_title: title,
                        published_at: "2026-04-09T00:00:00Z",
                    },
                    &[FtsChunk {
                        chunk_id: format!("{video_id}_transcript_0"),
                        section_title: None,
                        chunk_text: "Claude appears in this indexed transcript.".to_string(),
                        start_sec: None,
                    }],
                )
                .await
                .expect("fts source should be indexed");
        }

        let response = search(
            State(state),
            Extension(AccessContext {
                user_id: Some("user-1".to_string()),
                auth_state: AuthState::Authenticated,
                access_role: AccessRole::User,
                allowed_channel_ids: vec!["channel-a".to_string()],
                allowed_other_video_ids: vec!["video-other".to_string()],
            }),
            Query(SearchParams {
                q: "claude".to_string(),
                source: None,
                limit: Some(10),
                channel_id: None,
                mode: Some(SearchExecutionMode::Keyword),
            }),
        )
        .await
        .expect("search should succeed")
        .into_response();

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should read");
        let payload: SearchResponsePayload =
            serde_json::from_slice(&body).expect("payload should deserialize");
        let video_ids = payload
            .results
            .iter()
            .map(|result| result.video_id.clone())
            .collect::<HashSet<_>>();

        assert!(
            video_ids.contains("video-channel"),
            "channel-scoped result should be returned"
        );
        assert!(
            video_ids.contains("video-other"),
            "explicitly allowed other video should be returned"
        );
        assert!(
            !video_ids.contains("video-forbidden"),
            "out-of-scope video should be filtered out"
        );
    }
}
