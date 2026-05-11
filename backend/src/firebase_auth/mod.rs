use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use axum::http::HeaderValue;
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, dangerous::insecure_decode, decode, decode_header,
};
use reqwest::header::CACHE_CONTROL;
use serde::Deserialize;

use crate::config::SecurityRuntimeConfig;

fn extract_bearer_token(authorization: Option<&HeaderValue>) -> Result<Option<&str>, String> {
    let Some(header_value) = authorization else {
        return Ok(None);
    };

    let raw = header_value
        .to_str()
        .map_err(|_| "invalid Authorization header encoding".to_string())?
        .trim();
    if raw.is_empty() {
        return Err("Authorization header must not be empty".to_string());
    }

    let mut parts = raw.splitn(2, ' ');
    let scheme = parts.next().unwrap_or_default();
    let token = parts.next().unwrap_or_default().trim();

    if !scheme.eq_ignore_ascii_case("Bearer") || token.is_empty() {
        return Err("Authorization header must use Bearer authentication".to_string());
    }

    Ok(Some(token))
}

async fn get_securetoken_key(kid: &str) -> Result<Option<SecureTokenJwk>, String> {
    if let Some(cached_key) = {
        let cache = CACHED_SECURETOKEN_KEYS
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        cache.as_ref().and_then(|cached| {
            (cached.expires_at > Instant::now())
                .then(|| cached.keys_by_id.get(kid).cloned())
                .flatten()
        })
    } {
        return Ok(Some(cached_key));
    }

    let response = reqwest::get(GOOGLE_SECURETOKEN_JWKS_URL)
        .await
        .map_err(|error| format!("failed to fetch Firebase signing keys: {error}"))?;
    let ttl = response
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_cache_control_max_age)
        .unwrap_or(DEFAULT_SECURETOKEN_KEY_TTL);
    let jwk_set = response
        .json::<SecureTokenJwkSet>()
        .await
        .map_err(|error| format!("failed to parse Firebase signing keys: {error}"))?;

    let cached = CachedSecureTokenKeys {
        keys_by_id: jwk_set
            .keys
            .into_iter()
            .map(|key| (key.kid.clone(), key))
            .collect(),
        expires_at: Instant::now() + ttl,
    };
    let resolved = cached.keys_by_id.get(kid).cloned();

    let mut cache = CACHED_SECURETOKEN_KEYS
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    *cache = Some(cached);

    Ok(resolved)
}

async fn verify_signed_token(
    token: &str,
    config: &SecurityRuntimeConfig,
) -> Result<FirebaseClaims, String> {
    let header = decode_header(token)
        .map_err(|error| format!("failed to decode Firebase token header: {error}"))?;
    let kid = header
        .kid
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Firebase token is missing key id".to_string())?;

    let jwk = get_securetoken_key(kid)
        .await?
        .ok_or_else(|| format!("Firebase token key `{kid}` was not found in Google's key set"))?;
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|error| format!("failed to construct Firebase decoding key: {error}"))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_required_spec_claims(&["exp", "aud", "iss", "sub"]);
    validation.set_audience(&[config.firebase_project_id.as_str()]);
    validation.set_issuer(&[format!(
        "https://securetoken.google.com/{}",
        config.firebase_project_id
    )]);

    decode::<FirebaseClaims>(token, &decoding_key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|error| format!("failed to verify Firebase token: {error}"))
}

fn parse_cache_control_max_age(cache_control: &str) -> Option<Duration> {
    cache_control.split(',').find_map(|segment| {
        let trimmed = segment.trim();
        let value = trimmed.strip_prefix("max-age=")?;
        value.parse::<u64>().ok().map(Duration::from_secs)
    })
}

fn validate_claims(claims: &FirebaseClaims, config: &SecurityRuntimeConfig) -> Result<(), String> {
    if claims.sub.trim().is_empty() {
        return Err("Firebase token subject must not be empty".to_string());
    }

    if claims.aud != config.firebase_project_id {
        return Err("Firebase token audience did not match this app".to_string());
    }

    let expected_issuer = format!(
        "https://securetoken.google.com/{}",
        config.firebase_project_id
    );
    if claims.iss != expected_issuer {
        return Err("Firebase token issuer did not match this app".to_string());
    }

    Ok(())
}

fn build_verified_identity(claims: FirebaseClaims) -> VerifiedClientIdentity {
    let sign_in_provider = claims
        .firebase
        .as_ref()
        .and_then(|firebase| firebase.sign_in_provider.as_deref());

    if matches!(sign_in_provider, Some("anonymous")) {
        return VerifiedClientIdentity {
            user_id: None,
            email: None,
            auth_state: FirebaseAuthState::Anonymous,
        };
    }

    VerifiedClientIdentity {
        user_id: claims.user_id.or(Some(claims.sub)),
        email: claims.email,
        auth_state: FirebaseAuthState::Authenticated,
    }
}

pub async fn verify_bearer_identity(
    authorization: Option<&HeaderValue>,
    config: &SecurityRuntimeConfig,
) -> Result<Option<VerifiedClientIdentity>, String> {
    let Some(token) = extract_bearer_token(authorization)? else {
        return Ok(None);
    };

    let claims = if std::env::var("FIREBASE_AUTH_EMULATOR_HOST")
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        insecure_decode::<FirebaseClaims>(token)
            .map_err(|error| format!("failed to decode Firebase emulator token: {error}"))?
            .claims
    } else {
        verify_signed_token(token, config).await?
    };

    validate_claims(&claims, config)?;
    Ok(Some(build_verified_identity(claims)))
}

const GOOGLE_SECURETOKEN_JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
const DEFAULT_SECURETOKEN_KEY_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirebaseAuthState {
    Anonymous,
    Authenticated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedClientIdentity {
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub auth_state: FirebaseAuthState,
}

#[derive(Debug, Clone, Deserialize)]
struct FirebaseClaims {
    aud: String,
    iss: String,
    sub: String,
    user_id: Option<String>,
    email: Option<String>,
    firebase: Option<FirebaseSignInContext>,
}

#[derive(Debug, Clone, Deserialize)]
struct FirebaseSignInContext {
    sign_in_provider: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SecureTokenJwkSet {
    keys: Vec<SecureTokenJwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct SecureTokenJwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Clone)]
struct CachedSecureTokenKeys {
    keys_by_id: HashMap<String, SecureTokenJwk>,
    expires_at: Instant,
}

static CACHED_SECURETOKEN_KEYS: OnceLock<Mutex<Option<CachedSecureTokenKeys>>> = OnceLock::new();

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
