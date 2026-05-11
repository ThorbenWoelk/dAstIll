use std::{collections::HashSet, sync::Arc};

use axum::{
    Extension,
    body::to_bytes,
    extract::{Query, State},
    response::IntoResponse,
};
use reqwest::Client;
use tokio::sync::RwLock;

use super::{
    SearchExecutionMode, SearchParams, candidate_has_exact_query_signal,
    exact_keyword_rescue_candidates, promote_hybrid_keyword_rescue_results, search,
    should_promote_hybrid_keyword_rescue, should_rescue_semantic_results,
};
use crate::{
    models::SearchResponsePayload,
    models::SearchVideoResultPayload,
    models::{ContentItemKind, ContentSourceKind, ProviderKind},
    search::fts::FtsSourceMeta,
    search::{FtsChunk, SearchProgress, SearchService, SearchSourceKind},
    security::{AccessContext, AccessRole, AuthState},
    services::{
        ChatService, CloudCooldown, InputGuardrailService, OllamaCore, OpenAlexPlannerService,
        OpenAlexService, PodcastFeedService, SummarizerService, SummaryEvaluatorService,
        TranscriptCooldown, TranscriptService, UserActivity, WebsiteService, YouTubeQuotaCooldown,
        YouTubeService,
    },
    state::AppState,
};

fn search_candidate(video_title: &str, chunk_text: &str) -> crate::search::SearchCandidate {
    crate::search::SearchCandidate {
        chunk_id: video_title.to_string(),
        video_id: video_title.to_string(),
        channel_id: "channel".to_string(),
        channel_name: "Channel".to_string(),
        video_title: video_title.to_string(),
        source_kind: SearchSourceKind::Summary,
        section_title: None,
        chunk_text: chunk_text.to_string(),
        published_at: "2026-04-09T00:00:00Z".to_string(),
        start_sec: None,
    }
}

fn search_result(video_id: &str, video_title: &str) -> SearchVideoResultPayload {
    SearchVideoResultPayload {
        source_id: "source".to_string(),
        video_id: video_id.to_string(),
        item_id: video_id.to_string(),
        provider: ProviderKind::YouTube,
        source_kind: ContentSourceKind::YouTubeChannel,
        item_kind: ContentItemKind::Video,
        channel_id: "channel".to_string(),
        channel_name: "Channel".to_string(),
        video_title: video_title.to_string(),
        published_at: "2026-04-09T00:00:00Z".to_string(),
        matches: Vec::new(),
    }
}

#[test]
fn candidate_exact_query_signal_handles_conversational_phrase_queries() {
    let candidate = search_candidate(
        "Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing",
        "One Good Thing closes the episode.",
    );

    assert!(candidate_has_exact_query_signal(
        &candidate,
        "video where they talk about one good thing"
    ));
}

#[test]
fn candidate_exact_query_signal_can_combine_title_and_snippet_terms() {
    let candidate = search_candidate(
        "A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming",
        "Meta is developing an AI clone of Mark Zuckerberg for internal use.",
    );

    assert!(candidate_has_exact_query_signal(
        &candidate,
        "mark zuckerberg bot"
    ));
}

#[test]
fn semantic_results_rescue_when_exact_keyword_hit_exists_but_dense_results_do_not() {
    let exact_keyword_hit = search_candidate(
        "Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing",
        "One Good Thing closes the episode.",
    );
    let semantic_candidates = vec![
        search_candidate("This is good, actually", "A positive reaction video."),
        search_candidate("React feels insane", "A video about frontend frustration."),
    ];

    let keyword_rescue_candidates = exact_keyword_rescue_candidates(
        "video where they talk about one good thing",
        &[exact_keyword_hit],
    );

    assert!(should_rescue_semantic_results(
        "video where they talk about one good thing",
        &keyword_rescue_candidates,
        &semantic_candidates,
    ));
}

#[test]
fn semantic_results_rescue_when_combined_keyword_fields_cover_alias_like_queries() {
    let exact_keyword_hit = search_candidate(
        "A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming",
        "Meta is developing an AI clone of Mark Zuckerberg for internal use.",
    );
    let semantic_candidates = vec![
        search_candidate(
            "Clawdbot has gone rogue (I can't believe this is real)",
            "An OpenClaw bot experiment.",
        ),
        search_candidate(
            "Meta's weird plan to win the AI war",
            "Meta is chasing superintelligence under Zuckerberg.",
        ),
    ];

    let keyword_rescue_candidates =
        exact_keyword_rescue_candidates("mark zuckerberg bot", &[exact_keyword_hit]);

    assert!(should_rescue_semantic_results(
        "mark zuckerberg bot",
        &keyword_rescue_candidates,
        &semantic_candidates,
    ));
}

#[test]
fn hybrid_results_promote_exact_keyword_rescue_candidates() {
    let mut exact_keyword_hit = search_candidate(
        "Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing",
        "A New Yorker story about Sam Altman.",
    );
    exact_keyword_hit.video_id = "hard-fork-story".to_string();
    let keyword_rescue_candidates = vec![exact_keyword_hit];
    let results = vec![
        search_result("i-asked-sam", "I asked Sam Altman about the future of code"),
        search_result(
            "hard-fork-story",
            "Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing",
        ),
        search_result(
            "openai-files",
            "Is Sam Altman evil? The OpenAI Files are wild",
        ),
    ];

    assert!(should_promote_hybrid_keyword_rescue(
        &keyword_rescue_candidates,
        &results,
    ));

    let promoted = promote_hybrid_keyword_rescue_results(results, &keyword_rescue_candidates);

    assert_eq!(promoted[0].video_id, "hard-fork-story");
}

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
            crate::search::SEARCH_EMBEDDING_DIMENSIONS,
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
            crate::search::SEARCH_EMBEDDING_DIMENSIONS,
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
        fts: Arc::new(crate::search::FtsIndex::new().await.expect("fts index")),
        anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
        mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
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

#[tokio::test]
async fn keyword_search_relaxes_multi_term_paraphrases_when_exact_title_is_close() {
    let state = test_app_state().await;

    for (video_id, title, text) in [
        (
            "video-target",
            "Open source is dying",
            "AI is damaging open source through spam and degraded maintainer trust.",
        ),
        (
            "video-now",
            "Which browser should you use right now?",
            "A browser roundup about current recommendations right now.",
        ),
        (
            "video-dead",
            "Corepack is dead, and I'm scared",
            "Corepack is dead after npm registry changes and the ecosystem is nervous.",
        ),
    ] {
        state
            .fts
            .upsert_source(
                FtsSourceMeta {
                    video_id,
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-a",
                    channel_name: "Channel",
                    video_title: title,
                    published_at: "2026-04-09T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: format!("{video_id}_summary_0"),
                    section_title: None,
                    chunk_text: text.to_string(),
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
            allowed_other_video_ids: Vec::new(),
        }),
        Query(SearchParams {
            q: "open source is dead now".to_string(),
            source: None,
            limit: Some(5),
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

    assert_eq!(payload.results[0].video_title, "Open source is dying");
}
