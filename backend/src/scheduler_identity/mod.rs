//! Verifies that a request comes from Cloud Scheduler.
//!
//! Cloud Scheduler signs an OIDC ID token as a dedicated service account.
//! The token is checked against Google's public signing keys, the expected
//! audience, Google's issuer, and the configured service account email.

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use axum::http::HeaderValue;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::header::CACHE_CONTROL;
use serde::Deserialize;

use crate::config::CatchUpRuntimeConfig;
use crate::firebase_auth::{extract_bearer_token, parse_cache_control_max_age};

const GOOGLE_OIDC_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const GOOGLE_ISSUERS: [&str; 2] = ["https://accounts.google.com", "accounts.google.com"];
const DEFAULT_KEY_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Deserialize)]
pub(crate) struct SchedulerClaims {
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

#[derive(Clone, Deserialize)]
struct GoogleJwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct GoogleJwkSet {
    keys: Vec<GoogleJwk>,
}

struct CachedKeys {
    keys_by_id: HashMap<String, GoogleJwk>,
    expires_at: Instant,
}

static CACHED_KEYS: OnceLock<Mutex<Option<CachedKeys>>> = OnceLock::new();

/// Accepts the request only when it carries a valid Cloud Scheduler token.
pub async fn verify_scheduler_request(
    authorization: Option<&HeaderValue>,
    config: &CatchUpRuntimeConfig,
) -> Result<(), String> {
    let token = extract_bearer_token(authorization)?
        .ok_or_else(|| "missing scheduler bearer token".to_string())?;
    let claims = verify_signed_token(token, &config.audience).await?;
    check_scheduler_identity(&claims, config)
}

/// The token must belong to the configured, verified service account.
pub(crate) fn check_scheduler_identity(
    claims: &SchedulerClaims,
    config: &CatchUpRuntimeConfig,
) -> Result<(), String> {
    if claims.email_verified != Some(true) {
        return Err("scheduler token email is not verified".to_string());
    }
    match claims.email.as_deref() {
        Some(email) if email.eq_ignore_ascii_case(&config.invoker_email) => Ok(()),
        _ => Err("scheduler token was issued for another account".to_string()),
    }
}

async fn verify_signed_token(token: &str, audience: &str) -> Result<SchedulerClaims, String> {
    let header =
        decode_header(token).map_err(|error| format!("invalid scheduler token header: {error}"))?;
    let kid = header
        .kid
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "scheduler token is missing key id".to_string())?;
    let jwk = google_signing_key(&kid)
        .await?
        .ok_or_else(|| format!("scheduler token key `{kid}` is not a Google key"))?;
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|error| format!("invalid Google signing key: {error}"))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_required_spec_claims(&["exp", "aud", "iss"]);
    validation.set_audience(&[audience]);
    validation.set_issuer(&GOOGLE_ISSUERS);

    decode::<SchedulerClaims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|error| format!("scheduler token rejected: {error}"))
}

async fn google_signing_key(kid: &str) -> Result<Option<GoogleJwk>, String> {
    {
        let cache = CACHED_KEYS
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(cached) = cache.as_ref()
            && cached.expires_at > Instant::now()
            && let Some(key) = cached.keys_by_id.get(kid)
        {
            return Ok(Some(key.clone()));
        }
    }

    let response = reqwest::get(GOOGLE_OIDC_JWKS_URL)
        .await
        .map_err(|error| format!("failed to fetch Google signing keys: {error}"))?;
    let ttl = response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_cache_control_max_age)
        .unwrap_or(DEFAULT_KEY_TTL);
    let key_set = response
        .json::<GoogleJwkSet>()
        .await
        .map_err(|error| format!("failed to parse Google signing keys: {error}"))?;

    let keys_by_id: HashMap<String, GoogleJwk> = key_set
        .keys
        .into_iter()
        .map(|key| (key.kid.clone(), key))
        .collect();
    let resolved = keys_by_id.get(kid).cloned();
    *CACHED_KEYS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(CachedKeys {
        keys_by_id,
        expires_at: Instant::now() + ttl,
    });
    Ok(resolved)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
