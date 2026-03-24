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
pub const OPERATOR_ROLE: &str = "operator";
pub const PROXY_AUTH_HEADER: &str = "x-dastill-proxy-auth";
pub const ROLE_HEADER: &str = "x-dastill-role";
const X_FORWARDED_FOR_HEADER: &str = "x-forwarded-for";
const ANONYMOUS_CHAT_QUOTA_MESSAGE: &str = "Anonymous chat quota exceeded";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessRole {
    User,
    Operator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedRequest {
    pub role: AccessRole,
    pub client_key: String,
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

fn resolve_role(headers: &axum::http::HeaderMap) -> AccessRole {
    match headers
        .get(ROLE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
    {
        Some(OPERATOR_ROLE) => AccessRole::Operator,
        _ => AccessRole::User,
    }
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
    let client_key = request
        .extensions()
        .get::<AuthorizedRequest>()
        .map(|authorized| authorized.client_key.clone())
        .unwrap_or_else(|| extract_client_key(request.headers()));

    state
        .request_rate_limiter
        .enforce(tier, &client_key, Instant::now())
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

    let role = resolve_role(request.headers());
    let client_key = extract_client_key(request.headers());
    request
        .extensions_mut()
        .insert(AuthorizedRequest { role, client_key });

    next.run(request).await
}

pub async fn require_operator_role(request: Request, next: Next) -> Response {
    let is_operator = request
        .extensions()
        .get::<AuthorizedRequest>()
        .map(|authorized| authorized.role == AccessRole::Operator)
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
        .get::<AuthorizedRequest>()
        .cloned()
        .unwrap_or_else(|| AuthorizedRequest {
            role: resolve_role(request.headers()),
            client_key: extract_client_key(request.headers()),
        });

    if authorized.role == AccessRole::Operator {
        return next.run(request).await;
    }

    match consume_anonymous_chat_quota(&state, &authorized.client_key).await {
        Ok(true) => next.run(request).await,
        Ok(false) => build_anonymous_chat_quota_response(),
        Err(error) => {
            tracing::error!(
                error = %error,
                client_key_hash = %hash_client_key(&authorized.client_key),
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
    use super::{RateLimitTier, RequestRateLimiter};
    use crate::config::SecurityRuntimeConfig;

    #[test]
    fn request_rate_limiter_blocks_after_limit_is_reached() {
        let config = SecurityRuntimeConfig {
            proxy_token: "test".to_string(),
            allowed_origins: vec![],
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
}
