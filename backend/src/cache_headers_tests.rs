use super::cache_control_for_path;

// ── Health ──────────────────────────────────────────────────────────────

#[test]
fn health_endpoint_returns_no_store() {
    assert_eq!(cache_control_for_path("/api/health"), Some("no-store"));
}

#[test]
fn health_ai_endpoint_returns_no_store() {
    assert_eq!(cache_control_for_path("/api/health/ai"), Some("no-store"));
}

// ── Search ──────────────────────────────────────────────────────────────

#[test]
fn search_endpoint_returns_no_store() {
    assert_eq!(cache_control_for_path("/api/search"), Some("no-store"));
}

#[test]
fn search_status_endpoint_returns_no_store() {
    assert_eq!(
        cache_control_for_path("/api/search/status"),
        Some("no-store")
    );
}

#[test]
fn search_rebuild_endpoint_returns_no_store() {
    // POST in practice, but path-based routing assigns no-store regardless.
    assert_eq!(
        cache_control_for_path("/api/search/rebuild"),
        Some("no-store")
    );
}

// ── SSE streams ─────────────────────────────────────────────────────────

#[test]
fn search_stream_endpoint_returns_no_header() {
    assert_eq!(cache_control_for_path("/api/search/status/stream"), None);
}

#[test]
fn chat_stream_endpoint_returns_no_header() {
    assert_eq!(
        cache_control_for_path("/api/chat/conversations/conv-123/stream"),
        None
    );
}

// ── Channels ─────────────────────────────────────────────────────────────

#[test]
fn channels_list_returns_short_max_age() {
    assert_eq!(
        cache_control_for_path("/api/channels"),
        Some("max-age=10, stale-while-revalidate=30")
    );
}

#[test]
fn channel_detail_returns_moderate_max_age() {
    assert_eq!(
        cache_control_for_path("/api/channels/UCxxxx"),
        Some("max-age=60, stale-while-revalidate=300")
    );
}

#[test]
fn channel_snapshot_returns_short_max_age() {
    assert_eq!(
        cache_control_for_path("/api/channels/UCxxxx/snapshot"),
        Some("max-age=10, stale-while-revalidate=30")
    );
}

#[test]
fn channel_videos_returns_short_max_age() {
    assert_eq!(
        cache_control_for_path("/api/channels/UCxxxx/videos"),
        Some("max-age=10, stale-while-revalidate=30")
    );
}

#[test]
fn channel_sync_depth_returns_moderate_max_age() {
    assert_eq!(
        cache_control_for_path("/api/channels/UCxxxx/sync-depth"),
        Some("max-age=60, stale-while-revalidate=300")
    );
}

#[test]
fn channel_refresh_returns_no_header() {
    // POST endpoint.
    assert_eq!(cache_control_for_path("/api/channels/UCxxxx/refresh"), None);
}

#[test]
fn channel_backfill_returns_no_header() {
    // POST endpoint.
    assert_eq!(
        cache_control_for_path("/api/channels/UCxxxx/backfill"),
        None
    );
}

// ── Videos ───────────────────────────────────────────────────────────────

#[test]
fn video_detail_returns_moderate_max_age() {
    assert_eq!(
        cache_control_for_path("/api/videos/vid123"),
        Some("max-age=60, stale-while-revalidate=300")
    );
}

#[test]
fn video_transcript_returns_long_max_age() {
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/transcript"),
        Some("max-age=3600, stale-while-revalidate=86400")
    );
}

#[test]
fn video_summary_returns_moderate_max_age() {
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/summary"),
        Some("max-age=60, stale-while-revalidate=300")
    );
}

#[test]
fn video_info_returns_long_max_age() {
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/info"),
        Some("max-age=3600, stale-while-revalidate=86400")
    );
}

#[test]
fn video_highlights_returns_moderate_max_age() {
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/highlights"),
        Some("max-age=60, stale-while-revalidate=300")
    );
}

#[test]
fn video_ensure_returns_no_header() {
    // POST endpoints.
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/transcript/ensure"),
        None
    );
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/summary/ensure"),
        None
    );
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/info/ensure"),
        None
    );
}

// ── Workspace bootstrap ───────────────────────────────────────────────────

#[test]
fn workspace_bootstrap_returns_short_max_age() {
    assert_eq!(
        cache_control_for_path("/api/workspace/bootstrap"),
        Some("max-age=10, stale-while-revalidate=30")
    );
}

// ── Highlights ───────────────────────────────────────────────────────────

#[test]
fn highlights_list_returns_moderate_max_age() {
    assert_eq!(
        cache_control_for_path("/api/highlights"),
        Some("max-age=60, stale-while-revalidate=300")
    );
}

// ── Chat ─────────────────────────────────────────────────────────────────

#[test]
fn chat_config_returns_short_max_age() {
    assert_eq!(
        cache_control_for_path("/api/chat/config"),
        Some("max-age=10, stale-while-revalidate=30")
    );
}

#[test]
fn chat_conversations_list_returns_short_max_age() {
    assert_eq!(
        cache_control_for_path("/api/chat/conversations"),
        Some("max-age=10, stale-while-revalidate=30")
    );
}

#[test]
fn chat_conversation_detail_returns_short_max_age() {
    assert_eq!(
        cache_control_for_path("/api/chat/conversations/conv-123"),
        Some("max-age=10, stale-while-revalidate=30")
    );
}

#[test]
fn chat_messages_endpoint_returns_no_header() {
    // POST endpoint.
    assert_eq!(
        cache_control_for_path("/api/chat/conversations/conv-123/messages"),
        None
    );
}

#[test]
fn chat_cancel_endpoint_returns_no_header() {
    // POST endpoint.
    assert_eq!(
        cache_control_for_path("/api/chat/conversations/conv-123/cancel"),
        None
    );
}

// ── Edge cases ────────────────────────────────────────────────────────────

#[test]
fn path_with_query_string_strips_query_before_matching() {
    assert_eq!(
        cache_control_for_path("/api/channels?limit=20"),
        Some("max-age=10, stale-while-revalidate=30")
    );
    assert_eq!(
        cache_control_for_path("/api/health?check=true"),
        Some("no-store")
    );
    assert_eq!(
        cache_control_for_path("/api/videos/vid123/transcript?raw=true"),
        Some("max-age=3600, stale-while-revalidate=86400")
    );
}

#[test]
fn unknown_paths_return_no_header() {
    assert_eq!(cache_control_for_path("/api/unknown"), None);
    assert_eq!(cache_control_for_path("/"), None);
}

// ── Middleware status-gating ──────────────────────────────────────────────

use super::add_cache_control;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::get,
};
use tower::ServiceExt;

#[tokio::test]
async fn cache_control_header_added_to_2xx_get_response() {
    let app = Router::new()
        .route("/api/channels", get(|| async { StatusCode::OK }))
        .layer(middleware::from_fn(add_cache_control));

    let request = Request::builder()
        .method("GET")
        .uri("/api/channels")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers().contains_key("cache-control"),
        "Cache-Control header must be present on 2xx GET response"
    );
}

#[tokio::test]
async fn cache_control_header_not_added_to_4xx_get_response() {
    let app = Router::new()
        .route("/api/channels", get(|| async { StatusCode::NOT_FOUND }))
        .layer(middleware::from_fn(add_cache_control));

    let request = Request::builder()
        .method("GET")
        .uri("/api/channels")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(
        !response.headers().contains_key("cache-control"),
        "Cache-Control header must NOT be present on 4xx GET response"
    );
}

#[tokio::test]
async fn cache_control_header_not_added_to_5xx_get_response() {
    let app = Router::new()
        .route(
            "/api/channels",
            get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
        )
        .layer(middleware::from_fn(add_cache_control));

    let request = Request::builder()
        .method("GET")
        .uri("/api/channels")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        !response.headers().contains_key("cache-control"),
        "Cache-Control header must NOT be present on 5xx GET response"
    );
}

#[tokio::test]
async fn cache_control_header_not_added_to_post_response() {
    // POST requests must never get Cache-Control headers, regardless of status.
    use axum::routing::post;

    let app = Router::new()
        .route("/api/channels", post(|| async { StatusCode::OK }))
        .layer(middleware::from_fn(add_cache_control));

    let request = Request::builder()
        .method("POST")
        .uri("/api/channels")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        !response.headers().contains_key("cache-control"),
        "Cache-Control header must NOT be present on POST response"
    );
}
