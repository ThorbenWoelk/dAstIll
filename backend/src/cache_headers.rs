use axum::{
    extract::Request,
    http::{HeaderValue, Method, header},
    middleware::Next,
    response::Response,
};

const SHORT: &str = "max-age=10, stale-while-revalidate=30";
const MODERATE: &str = "max-age=60, stale-while-revalidate=300";
const LONG: &str = "max-age=3600, stale-while-revalidate=86400";
const NO_STORE: &str = "no-store";

/// Determine the `Cache-Control` header value for a given request path.
///
/// Returns `None` when no caching header should be added (e.g. SSE streams,
/// or paths that are not explicitly cacheable).
pub(crate) fn cache_control_for_path(path: &str) -> Option<&'static str> {
    // Strip query string before matching.
    let path = path.split('?').next().unwrap_or(path);

    // SSE streams: never add cache headers.
    if path.ends_with("/stream") {
        return None;
    }

    // Health endpoints: always no-store.
    if path == "/api/health" || path.starts_with("/api/health/") {
        return Some(NO_STORE);
    }

    // Search: dynamic/volatile content, no-store.
    if path == "/api/search" || path.starts_with("/api/search/") {
        return Some(NO_STORE);
    }

    // Video sub-resources — check before the exact /api/videos/{id} case.
    if path.starts_with("/api/videos/") {
        if path.ends_with("/transcript") {
            // Immutable once created.
            return Some(LONG);
        }
        if path.ends_with("/summary") {
            return Some(MODERATE);
        }
        if path.ends_with("/info") {
            // Stable once fetched from YouTube.
            return Some(LONG);
        }
        if path.ends_with("/highlights") {
            return Some(MODERATE);
        }
        // /api/videos/{id} (exactly 3 path segments).
        let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if segments.len() == 3 {
            return Some(MODERATE);
        }
        // Other video sub-paths (/ensure, /clean, etc.) are POST — no header.
        return None;
    }

    // Channel sub-resources — check before the exact /api/channels/{id} case.
    if path.starts_with("/api/channels/") {
        if path.ends_with("/snapshot") || path.ends_with("/videos") {
            return Some(SHORT);
        }
        if path.ends_with("/sync-depth") {
            return Some(MODERATE);
        }
        // /api/channels/{id} (exactly 3 path segments).
        let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if segments.len() == 3 {
            return Some(MODERATE);
        }
        // Other channel sub-paths (/refresh, /backfill) are POST — no header.
        return None;
    }

    // Top-level list / aggregate endpoints.
    if path == "/api/channels" {
        return Some(SHORT);
    }
    if path == "/api/workspace/bootstrap" {
        return Some(SHORT);
    }
    if path == "/api/highlights" {
        return Some(MODERATE);
    }

    // Chat conversations.
    if path == "/api/chat/conversations" {
        return Some(SHORT);
    }
    if path.starts_with("/api/chat/conversations/") {
        // /api/chat/conversations/{id} = 4 path segments.
        let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if segments.len() == 4 {
            return Some(SHORT);
        }
        // Deeper sub-paths (/messages, /cancel) are POST/DELETE — no header.
        return None;
    }

    None
}

/// Axum middleware that injects `Cache-Control` headers into GET responses.
///
/// Only GET responses for known cacheable endpoints receive a header.
/// POST, PUT, and DELETE responses are never touched.
pub async fn add_cache_control(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let mut response = next.run(request).await;

    if method == Method::GET {
        if let Some(value) = cache_control_for_path(&path) {
            if let Ok(header_value) = HeaderValue::from_str(value) {
                response
                    .headers_mut()
                    .insert(header::CACHE_CONTROL, header_value);
            }
        }
    }

    response
}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            cache_control_for_path("/api/search/status/stream"),
            None
        );
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
        assert_eq!(
            cache_control_for_path("/api/channels/UCxxxx/refresh"),
            None
        );
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
}
