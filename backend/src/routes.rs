use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use tower_http::trace::TraceLayer;

use crate::cache_headers::add_cache_control;
use crate::config::SecurityRuntimeConfig;
use crate::handlers::{
    analytics, auth, channels, chat, content, highlights, library, mini, preferences, videos,
};
use crate::search::handler as search;
use crate::security::{
    build_cors_layer, enforce_anonymous_chat_quota, enforce_baseline_rate_limit,
    enforce_expensive_rate_limit, require_operator_role, require_proxy_auth,
};
use crate::state::AppState;

pub fn build_app(
    state: AppState,
    security_runtime: &SecurityRuntimeConfig,
) -> anyhow::Result<Router> {
    let protected_api = Router::new()
        .route("/api/health/ai", get(content::health_ai))
        .route("/api/chat/config", get(chat::get_client_config))
        .route(
            "/api/chat/suggestions/channels",
            get(chat::suggest_channels),
        )
        .route("/api/chat/suggestions/videos", get(chat::suggest_videos))
        .route(
            "/api/chat/conversations",
            get(chat::list_conversations)
                .post(chat::create_conversation)
                .delete(chat::delete_all_conversations),
        )
        .route(
            "/api/chat/conversations/{id}",
            get(chat::get_conversation)
                .put(chat::update_conversation)
                .delete(chat::delete_conversation),
        )
        .route(
            "/api/chat/ephemeral/messages",
            post(chat::start_ephemeral_reply)
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_expensive_rate_limit,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_anonymous_chat_quota,
                )),
        )
        .route(
            "/api/chat/conversations/{id}/messages",
            post(chat::start_conversation_reply)
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_expensive_rate_limit,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_anonymous_chat_quota,
                )),
        )
        .route(
            "/api/chat/conversations/{id}/stream",
            get(chat::resume_conversation_reply).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/chat/conversations/{id}/cancel",
            post(chat::cancel_conversation_reply),
        )
        .route(
            "/api/preferences",
            get(preferences::get_preferences).put(preferences::save_preferences),
        )
        .route("/api/search", get(search::search))
        .route("/api/search/status", get(search::search_status))
        .route(
            "/api/search/status/stream",
            get(search::search_status_stream).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/search/rebuild",
            post(search::rebuild_search_projection)
                .layer(middleware::from_fn(require_operator_role))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_expensive_rate_limit,
                )),
        )
        .route(
            "/api/workspace/bootstrap",
            get(channels::workspace_bootstrap),
        )
        .route(
            "/api/library/website-folders",
            get(library::list_website_folders).post(library::create_website_folder),
        )
        .route(
            "/api/library/website-folders/reorder",
            post(library::reorder_website_folders),
        )
        .route(
            "/api/library/website-folders/{id}",
            axum::routing::put(library::update_website_folder)
                .delete(library::delete_website_folder),
        )
        .route(
            "/api/auth/mobile-handoff",
            post(auth::create_mobile_auth_handoff),
        )
        .route(
            "/api/auth/mobile-handoff/{id}",
            axum::routing::put(auth::complete_mobile_auth_handoff),
        )
        .route(
            "/api/auth/mobile-handoff/{id}/redeem",
            post(auth::redeem_mobile_auth_handoff),
        )
        .route("/api/mini", get(mini::get_mini_reader))
        .route(
            "/api/mini/videos/{id}/read",
            axum::routing::put(mini::update_mini_read_status),
        )
        .route(
            "/api/channels",
            get(channels::list_channels).post(channels::add_channel),
        )
        .route(
            "/api/openalex/plan",
            post(channels::plan_openalex_query).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/channels/{id}",
            get(channels::get_channel)
                .put(channels::update_channel)
                .delete(channels::delete_channel),
        )
        .route(
            "/api/channels/{id}/sync-depth",
            get(channels::get_channel_sync_depth),
        )
        .route(
            "/api/channels/{id}/snapshot",
            get(channels::get_channel_snapshot),
        )
        .route(
            "/api/channels/{id}/refresh",
            post(channels::refresh_channel_videos).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/channels/{id}/backfill",
            post(channels::backfill_channel_videos).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/channels/{id}/videos",
            get(videos::list_channel_videos),
        )
        .route("/api/videos", post(videos::add_manual_video))
        .route("/api/videos/{id}", get(videos::get_video))
        .route("/api/videos/{id}/info", get(videos::get_video_info))
        .route(
            "/api/videos/{id}/info/ensure",
            post(videos::ensure_video_info).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/info/backfill",
            post(videos::backfill_video_info).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route("/api/videos/{id}/transcript", get(content::get_transcript))
        .route(
            "/api/videos/{id}/transcript/ensure",
            post(content::generate_transcript).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/{id}/transcript",
            axum::routing::put(content::update_transcript),
        )
        .route(
            "/api/videos/{id}/acknowledged",
            axum::routing::put(videos::update_video_acknowledged),
        )
        .route(
            "/api/videos/{id}/transcript/clean",
            post(content::clean_transcript_formatting).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route("/api/videos/{id}/summary", get(content::get_summary))
        .route(
            "/api/videos/{id}/summary/audio",
            get(content::get_summary_audio)
                .post(content::generate_summary_audio)
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    enforce_expensive_rate_limit,
                )),
        )
        .route(
            "/api/videos/{id}/summary/audio/debug",
            get(content::get_summary_audio_debug).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/{id}/summary/ensure",
            post(content::generate_summary).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/{id}/summary",
            axum::routing::put(content::update_summary),
        )
        .route(
            "/api/videos/{id}/summary/regenerate",
            post(content::regenerate_summary).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route(
            "/api/videos/{id}/reset",
            post(content::reset_video).layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_expensive_rate_limit,
            )),
        )
        .route("/api/highlights", get(highlights::list_highlights))
        .route(
            "/api/videos/{id}/highlights",
            get(highlights::list_video_highlights).post(highlights::create_highlight),
        )
        .route("/api/highlights/{id}", delete(highlights::delete_highlight))
        .route("/api/analytics/events", post(analytics::ingest_events))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            enforce_baseline_rate_limit,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_proxy_auth,
        ));

    Ok(Router::new()
        .route("/api/health", get(crate::openapi::health))
        .route("/api/openapi.json", get(crate::openapi::get_openapi_json))
        .merge(protected_api)
        .layer(middleware::from_fn(add_cache_control))
        .layer(build_cors_layer(security_runtime).map_err(|err| anyhow::anyhow!(err))?)
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}
