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

    // Chat client configuration (model id for UI).
    if path == "/api/chat/config" {
        return Some(SHORT);
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

    if method == Method::GET && response.status().is_success() {
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
#[path = "tests.rs"]
mod tests;
