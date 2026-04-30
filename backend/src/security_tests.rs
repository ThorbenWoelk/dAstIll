use axum::http::{HeaderMap, HeaderValue};

use super::{
    AUTH_STATE_HEADER, AccessContext, AccessRole, AuthState, CLIENT_IP_HEADER, OPERATOR_ROLE,
    ROLE_HEADER, RateLimitTier, RequestRateLimiter, USER_ID_HEADER, build_access_context,
    can_use_db_inspect,
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
