use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
    extract::{Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::SecurityRuntimeConfig;
use crate::state::AppState;

pub const CLIENT_IP_HEADER: &str = "x-dastill-client-ip";
pub const AUTH_STATE_HEADER: &str = "x-dastill-auth-state";
pub const OPERATOR_ROLE: &str = "operator";
pub const PROXY_AUTH_HEADER: &str = "x-dastill-proxy-auth";
pub const ROLE_HEADER: &str = "x-dastill-role";
pub const USER_ID_HEADER: &str = "x-dastill-user-id";
const AUTHENTICATED_AUTH_STATE: &str = "authenticated";
const X_FORWARDED_FOR_HEADER: &str = "x-forwarded-for";
const ANONYMOUS_CHAT_QUOTA_MESSAGE: &str = "Anonymous chat quota exceeded";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRole {
    Anonymous,
    User,
    Operator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthState {
    Anonymous,
    Authenticated,
}

impl AuthState {
    fn is_authenticated(self) -> bool {
        matches!(self, Self::Authenticated)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessContext {
    pub user_id: Option<String>,
    pub auth_state: AuthState,
    pub access_role: AccessRole,
    pub allowed_channel_ids: Vec<String>,
    pub allowed_other_video_ids: Vec<String>,
}

impl AccessContext {
    fn is_operator(&self) -> bool {
        self.access_role == AccessRole::Operator
    }

    pub fn cache_scope_key(&self) -> String {
        match (&self.user_id, self.auth_state) {
            (Some(user_id), AuthState::Authenticated) => format!("user:{user_id}"),
            _ => "anonymous".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnonymousChatQuotaRecord {
    count: u32,
    first_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RateLimitTier {
    Baseline,
    Expensive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RateLimitPolicy {
    max_requests: usize,
    window: Duration,
}

#[derive(Debug)]
pub struct RequestRateLimiter {
    baseline: RateLimitPolicy,
    expensive: RateLimitPolicy,
    buckets: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl RequestRateLimiter {
    pub fn new(config: &SecurityRuntimeConfig) -> Self {
        Self {
            baseline: RateLimitPolicy {
                max_requests: config.baseline_rate_limit_per_minute as usize,
                window: Duration::from_secs(60),
            },
            expensive: RateLimitPolicy {
                max_requests: config.expensive_rate_limit_per_minute as usize,
                window: Duration::from_secs(60),
            },
            buckets: Mutex::new(HashMap::new()),
        }
    }

    fn enforce(&self, tier: RateLimitTier, client_key: &str, now: Instant) -> Result<(), Duration> {
        let policy = match tier {
            RateLimitTier::Baseline => self.baseline,
            RateLimitTier::Expensive => self.expensive,
        };
        let bucket_key = format!("{}:{client_key}", tier.as_str());
        let mut buckets = self.buckets.lock().unwrap_or_else(|err| err.into_inner());
        let bucket = buckets.entry(bucket_key).or_default();

        while let Some(first_seen_at) = bucket.front().copied() {
            if now.duration_since(first_seen_at) >= policy.window {
                bucket.pop_front();
            } else {
                break;
            }
        }

        if bucket.len() >= policy.max_requests {
            let retry_after = bucket
                .front()
                .copied()
                .map(|first_seen_at| {
                    policy
                        .window
                        .saturating_sub(now.duration_since(first_seen_at))
                })
                .unwrap_or(policy.window);
            return Err(retry_after);
        }

        bucket.push_back(now);
        Ok(())
    }
}

impl RateLimitTier {
    fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Expensive => "expensive",
        }
    }
}

fn secure_equals(left: &str, right: &str) -> bool {
    let left_hash = Sha256::digest(left.as_bytes());
    let right_hash = Sha256::digest(right.as_bytes());
    left_hash.as_slice() == right_hash.as_slice()
}

fn extract_client_key(headers: &axum::http::HeaderMap) -> String {
    headers
        .get(CLIENT_IP_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            headers
                .get(HeaderName::from_static(X_FORWARDED_FOR_HEADER))
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(',').next())
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("unknown")
        .to_string()
}

fn extract_user_id(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(USER_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn resolve_auth_state(headers: &axum::http::HeaderMap) -> AuthState {
    match headers
        .get(AUTH_STATE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
    {
        Some(AUTHENTICATED_AUTH_STATE) => AuthState::Authenticated,
        _ => AuthState::Anonymous,
    }
}

pub fn scope_cache_key(access_context: &AccessContext) -> String {
    match access_context.auth_state {
        AuthState::Authenticated => access_context
            .user_id
            .as_deref()
            .map(|user_id| format!("user:{user_id}"))
            .unwrap_or_else(|| "user:missing".to_string()),
        AuthState::Anonymous => "anonymous".to_string(),
    }
}

pub fn can_access_channel(access_context: &AccessContext, channel_id: &str) -> bool {
    channel_id == crate::models::OTHERS_CHANNEL_ID
        || access_context.allowed_channel_ids.iter().any(|id| id == channel_id)
}

pub fn can_access_video(
    access_context: &AccessContext,
    video_id: &str,
    channel_id: &str,
) -> bool {
    access_context
        .allowed_channel_ids
        .iter()
        .any(|id| id == channel_id)
        || access_context
            .allowed_other_video_ids
            .iter()
            .any(|id| id == video_id)
}

fn resolve_proxy_identity(headers: &axum::http::HeaderMap) -> (AuthState, Option<String>) {
    match (resolve_auth_state(headers), extract_user_id(headers)) {
        (AuthState::Authenticated, Some(user_id)) => (AuthState::Authenticated, Some(user_id)),
        _ => (AuthState::Anonymous, None),
    }
}

fn resolve_access_role(headers: &axum::http::HeaderMap, auth_state: AuthState) -> AccessRole {
    if !auth_state.is_authenticated() {
        return AccessRole::Anonymous;
    }

    match headers
        .get(ROLE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
    {
        Some(OPERATOR_ROLE) => AccessRole::Operator,
        _ => AccessRole::User,
    }
}

fn resolve_allowed_channel_ids(
    auth_state: AuthState,
    default_seeded_channel_id: &str,
    authenticated_channel_ids: Vec<String>,
) -> Vec<String> {
    if auth_state.is_authenticated() {
        return authenticated_channel_ids;
    }

    vec![default_seeded_channel_id.to_string()]
}

fn resolve_allowed_other_video_ids(
    auth_state: AuthState,
    authenticated_video_ids: Vec<String>,
) -> Vec<String> {
    if auth_state.is_authenticated() {
        return authenticated_video_ids;
    }

    Vec::new()
}

fn build_access_context(
    headers: &axum::http::HeaderMap,
    default_seeded_channel_id: &str,
    authenticated_channel_ids: Vec<String>,
    authenticated_other_video_ids: Vec<String>,
) -> AccessContext {
    let (auth_state, user_id) = resolve_proxy_identity(headers);
    let access_role = resolve_access_role(headers, auth_state);
    let allowed_channel_ids = resolve_allowed_channel_ids(
        auth_state,
        default_seeded_channel_id,
        authenticated_channel_ids,
    );
    let allowed_other_video_ids =
        resolve_allowed_other_video_ids(auth_state, authenticated_other_video_ids);

    AccessContext {
        user_id,
        auth_state,
        access_role,
        allowed_channel_ids,
        allowed_other_video_ids,
    }
}

async fn load_authenticated_allowed_channel_ids(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<String>, String> {
    crate::db::ensure_user_seeded_channel_subscription(
        &state.db,
        user_id,
        &state.security.default_seeded_channel_id,
    )
    .await
    .map_err(|error| error.to_string())?;

    crate::db::migrate_legacy_preferences(&state.db, user_id)
        .await
        .map_err(|error| error.to_string())?;

    crate::db::list_user_channel_subscriptions(&state.db, user_id)
        .await
        .map(|channels| {
            channels
                .into_iter()
                .map(|channel| channel.channel_id)
                .collect()
        })
        .map_err(|error| error.to_string())
}

async fn load_authenticated_allowed_other_video_ids(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<String>, String> {
    crate::db::list_user_video_memberships(&state.db, user_id)
        .await
        .map(|memberships| memberships.into_iter().map(|entry| entry.video_id).collect())
        .map_err(|error| error.to_string())
}

async fn resolve_access_context(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<AccessContext, String> {
    let Some(user_id) = extract_user_id(headers).filter(|_| resolve_auth_state(headers).is_authenticated()) else {
        return Ok(build_access_context(
            headers,
            &state.security.default_seeded_channel_id,
            Vec::new(),
            Vec::new(),
        ));
    };

    let authenticated_channel_ids = load_authenticated_allowed_channel_ids(state, &user_id).await?;
    let authenticated_other_video_ids =
        load_authenticated_allowed_other_video_ids(state, &user_id).await?;

    Ok(build_access_context(
        headers,
        &state.security.default_seeded_channel_id,
        authenticated_channel_ids,
        authenticated_other_video_ids,
    ))
}

fn build_rate_limit_response(retry_after: Duration) -> Response {
    let mut response = (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    let retry_after_seconds = retry_after.as_secs().max(1).to_string();
    if let Ok(value) = HeaderValue::from_str(&retry_after_seconds) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    response
}

fn hash_client_key(client_key: &str) -> String {
    Sha256::digest(client_key.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn anonymous_chat_quota_key(client_key: &str) -> String {
    format!("chat-quota/{}.json", hash_client_key(client_key))
}

fn build_anonymous_chat_quota_response() -> Response {
    (StatusCode::FORBIDDEN, ANONYMOUS_CHAT_QUOTA_MESSAGE).into_response()
}

async fn consume_anonymous_chat_quota(state: &AppState, client_key: &str) -> Result<bool, String> {
    let _lock = state.anonymous_chat_quota_lock.lock().await;
    let conn = state.db.connect();
    let storage_key = anonymous_chat_quota_key(client_key);
    let now = Utc::now();
    let mut record = conn
        .get_json::<AnonymousChatQuotaRecord>(&storage_key)
        .await
        .map_err(|error| error.to_string())?
        .unwrap_or(AnonymousChatQuotaRecord {
            count: 0,
            first_seen_at: now,
        });

    // Reset the window if 24 hours have elapsed since first use in this period.
    if (now - record.first_seen_at).num_hours() >= 24 {
        record.count = 0;
        record.first_seen_at = now;
    }

    if record.count >= state.security.anonymous_chat_quota {
        return Ok(false);
    }

    record.count += 1;
    conn.put_json(&storage_key, &record)
        .await
        .map_err(|error| error.to_string())?;
    Ok(true)
}

fn enforce_rate_limit_for_request(
    state: &AppState,
    request: &Request,
    tier: RateLimitTier,
) -> Result<(), Box<Response>> {
    let rate_limit_key = request
        .extensions()
        .get::<AccessContext>()
        .and_then(|access_context| {
            access_context
                .auth_state
                .is_authenticated()
                .then(|| access_context.user_id.clone())
                .flatten()
        })
        .unwrap_or_else(|| extract_client_key(request.headers()));

    state
        .request_rate_limiter
        .enforce(tier, &rate_limit_key, Instant::now())
        .map_err(|error| Box::new(build_rate_limit_response(error)))
}

pub async fn require_proxy_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let is_authorized = request
        .headers()
        .get(PROXY_AUTH_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| secure_equals(value, &state.security.proxy_token))
        .unwrap_or(false);

    if !is_authorized {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let access_context = match resolve_access_context(&state, request.headers()).await {
        Ok(access_context) => access_context,
        Err(error) => {
            tracing::error!(error = %error, "failed to resolve request access context");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to resolve request access context",
            )
                .into_response();
        }
    };
    request.extensions_mut().insert(access_context);

    next.run(request).await
}

pub async fn require_operator_role(request: Request, next: Next) -> Response {
    let is_operator = request
        .extensions()
        .get::<AccessContext>()
        .map(AccessContext::is_operator)
        .unwrap_or(false);

    if !is_operator {
        return (StatusCode::FORBIDDEN, "Operator access required").into_response();
    }

    next.run(request).await
}

pub async fn enforce_anonymous_chat_quota(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let authorized = request
        .extensions()
        .get::<AccessContext>()
        .cloned()
        .unwrap_or_else(|| build_access_context(request.headers(), "", Vec::new(), Vec::new()));

    if authorized.auth_state.is_authenticated() {
        return next.run(request).await;
    }

    let client_key = extract_client_key(request.headers());
    match consume_anonymous_chat_quota(&state, &client_key).await {
        Ok(true) => next.run(request).await,
        Ok(false) => build_anonymous_chat_quota_response(),
        Err(error) => {
            tracing::error!(
                error = %error,
                client_key_hash = %hash_client_key(&client_key),
                "failed to enforce anonymous chat quota"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to enforce anonymous chat quota",
            )
                .into_response()
        }
    }
}

pub async fn enforce_baseline_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    match enforce_rate_limit_for_request(&state, &request, RateLimitTier::Baseline) {
        Ok(()) => next.run(request).await,
        Err(response) => *response,
    }
}

pub async fn enforce_expensive_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    match enforce_rate_limit_for_request(&state, &request, RateLimitTier::Expensive) {
        Ok(()) => next.run(request).await,
        Err(response) => *response,
    }
}

pub fn build_cors_layer(config: &SecurityRuntimeConfig) -> Result<CorsLayer, String> {
    let cors_layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE]);

    if config.allowed_origins.is_empty() {
        return Ok(cors_layer);
    }

    let allowed_origins = config
        .allowed_origins
        .iter()
        .map(|origin| {
            HeaderValue::from_str(origin).map_err(|err| {
                format!("invalid BACKEND_CORS_ALLOWED_ORIGINS value `{origin}`: {err}")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(cors_layer.allow_origin(AllowOrigin::list(allowed_origins)))
}

pub fn rate_limiter(config: &SecurityRuntimeConfig) -> Arc<RequestRateLimiter> {
    Arc::new(RequestRateLimiter::new(config))
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::{
        AUTH_STATE_HEADER, AccessContext, AccessRole, AuthState, CLIENT_IP_HEADER, OPERATOR_ROLE,
        ROLE_HEADER, RateLimitTier, RequestRateLimiter, USER_ID_HEADER, build_access_context,
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
            allowed_origins: vec![],
            default_seeded_channel_id: "seeded-channel".to_string(),
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
            allowed_origins: vec![],
            default_seeded_channel_id: "seeded-channel".to_string(),
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

        let access_context = build_access_context(&headers, "seeded-channel", Vec::new(), Vec::new());

        assert_eq!(
            access_context,
            AccessContext {
                user_id: None,
                auth_state: AuthState::Anonymous,
                access_role: AccessRole::Anonymous,
                allowed_channel_ids: vec!["seeded-channel".to_string()],
                allowed_other_video_ids: Vec::new(),
            }
        );
    }

    #[test]
    fn build_access_context_uses_authenticated_identity_and_subscriptions() {
        let mut headers = test_headers();
        headers.insert(
            AUTH_STATE_HEADER,
            HeaderValue::from_static("authenticated"),
        );
        headers.insert(USER_ID_HEADER, HeaderValue::from_static("firebase-uid-123"));
        headers.insert(ROLE_HEADER, HeaderValue::from_static(OPERATOR_ROLE));

        let access_context = build_access_context(
            &headers,
            "seeded-channel",
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
}
