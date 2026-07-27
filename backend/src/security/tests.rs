use axum::http::{HeaderMap, HeaderValue};

use super::{
    AUTH_STATE_HEADER, AccessContext, AccessRole, AuthState, CLIENT_IP_HEADER, OPERATOR_ROLE,
    ROLE_HEADER, RateLimitTier, RequestRateLimiter, USER_ID_HEADER, build_access_context,
    can_use_db_inspect, require_operator_role,
};
use crate::config::SecurityRuntimeConfig;

fn test_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CLIENT_IP_HEADER, HeaderValue::from_static("203.0.113.7"));
    headers
}

#[test]
fn request_rate_limiter_blocks_after_limit_is_reached() {
    let config = SecurityRuntimeConfig {
        proxy_token: "test".to_string(),
        firebase_project_id: "demo-dastill".to_string(),
        allowed_origins: vec![],
        operator_email_allowlist: vec![],
        default_seeded_channel_id: "seeded-channel".to_string(),
        default_seeded_channel_ids: vec!["seeded-channel".to_string()],
        baseline_rate_limit_per_minute: 2,
        expensive_rate_limit_per_minute: 1,
        anonymous_chat_quota: 10,
    };
    let limiter = RequestRateLimiter::new(&config);
    let now = std::time::Instant::now();

    assert!(
        limiter
            .enforce(RateLimitTier::Baseline, "client-1", now)
            .is_ok()
    );
    assert!(
        limiter
            .enforce(RateLimitTier::Baseline, "client-1", now)
            .is_ok()
    );
    assert!(
        limiter
            .enforce(RateLimitTier::Baseline, "client-1", now)
            .is_err()
    );
}

#[test]
fn request_rate_limiter_resets_after_window_expires() {
    let config = SecurityRuntimeConfig {
        proxy_token: "test".to_string(),
        firebase_project_id: "demo-dastill".to_string(),
        allowed_origins: vec![],
        operator_email_allowlist: vec![],
        default_seeded_channel_id: "seeded-channel".to_string(),
        default_seeded_channel_ids: vec!["seeded-channel".to_string()],
        baseline_rate_limit_per_minute: 1,
        expensive_rate_limit_per_minute: 1,
        anonymous_chat_quota: 10,
    };
    let limiter = RequestRateLimiter::new(&config);
    let start = std::time::Instant::now();

    assert!(
        limiter
            .enforce(RateLimitTier::Expensive, "client-2", start)
            .is_ok()
    );
    assert!(
        limiter
            .enforce(
                RateLimitTier::Expensive,
                "client-2",
                start + std::time::Duration::from_secs(61),
            )
            .is_ok()
    );
}

#[test]
fn build_access_context_uses_seeded_channel_for_anonymous_requests() {
    let headers = test_headers();

    let seeded_channel_ids = vec!["seeded-channel".to_string(), "podcast-seed".to_string()];
    let access_context =
        build_access_context(&headers, &seeded_channel_ids, Vec::new(), Vec::new());

    assert_eq!(
        access_context,
        AccessContext {
            user_id: None,
            auth_state: AuthState::Anonymous,
            access_role: AccessRole::Anonymous,
            allowed_channel_ids: seeded_channel_ids,
            allowed_other_video_ids: Vec::new(),
        }
    );
}

#[test]
fn build_access_context_uses_authenticated_identity_and_subscriptions() {
    let mut headers = test_headers();
    headers.insert(AUTH_STATE_HEADER, HeaderValue::from_static("authenticated"));
    headers.insert(USER_ID_HEADER, HeaderValue::from_static("firebase-uid-123"));
    headers.insert(ROLE_HEADER, HeaderValue::from_static(OPERATOR_ROLE));

    let access_context = build_access_context(
        &headers,
        &["seeded-channel".to_string()],
        vec!["channel-a".to_string(), "channel-b".to_string()],
        vec!["video-z".to_string()],
    );

    assert_eq!(
        access_context,
        AccessContext {
            user_id: Some("firebase-uid-123".to_string()),
            auth_state: AuthState::Authenticated,
            access_role: AccessRole::Operator,
            allowed_channel_ids: vec!["channel-a".to_string(), "channel-b".to_string()],
            allowed_other_video_ids: vec!["video-z".to_string()],
        }
    );
}

#[test]
fn db_inspect_is_available_for_read_only_queries() {
    assert!(can_use_db_inspect(&AccessContext {
        user_id: None,
        auth_state: AuthState::Anonymous,
        access_role: AccessRole::Anonymous,
        allowed_channel_ids: vec!["seeded-channel".to_string()],
        allowed_other_video_ids: Vec::new(),
    }));

    assert!(can_use_db_inspect(&AccessContext {
        user_id: Some("firebase-uid-123".to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::User,
        allowed_channel_ids: vec!["channel-a".to_string()],
        allowed_other_video_ids: Vec::new(),
    }));

    assert!(can_use_db_inspect(&AccessContext {
        user_id: Some("firebase-uid-999".to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::Operator,
        allowed_channel_ids: vec!["channel-a".to_string()],
        allowed_other_video_ids: Vec::new(),
    }));
}

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::post,
};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn access_context(role: AccessRole) -> AccessContext {
    AccessContext {
        user_id: Some("firebase-uid-123".to_string()),
        auth_state: AuthState::Authenticated,
        access_role: role,
        allowed_channel_ids: vec!["channel-a".to_string()],
        allowed_other_video_ids: Vec::new(),
    }
}

async fn post_with_access_context(role: AccessRole) -> (StatusCode, String) {
    let app = Router::new()
        .route(
            "/api/videos/info/backfill",
            post(|| async { StatusCode::OK }).layer(middleware::from_fn(require_operator_role)),
        )
        .layer(middleware::from_fn(
            move |mut request: Request<Body>, next: axum::middleware::Next| {
                let role = role;
                async move {
                    request.extensions_mut().insert(access_context(role));
                    next.run(request).await
                }
            },
        ));

    let request = Request::builder()
        .method("POST")
        .uri("/api/videos/info/backfill")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    (status, body)
}

#[tokio::test]
async fn require_operator_role_rejects_non_operator_video_info_backfill() {
    let (status, body) = post_with_access_context(AccessRole::User).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body, "Operator access required");

    let (status, body) = post_with_access_context(AccessRole::Anonymous).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body, "Operator access required");
}

#[tokio::test]
async fn require_operator_role_allows_operator_video_info_backfill() {
    let (status, body) = post_with_access_context(AccessRole::Operator).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_empty());
}
