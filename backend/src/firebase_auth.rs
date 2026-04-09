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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::http::HeaderValue;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::Serialize;

    use super::{
        FirebaseAuthState, VerifiedClientIdentity, build_verified_identity,
        parse_cache_control_max_age, verify_bearer_identity,
    };
    use crate::config::SecurityRuntimeConfig;

    #[derive(Serialize)]
    struct TestClaims {
        aud: String,
        iss: String,
        sub: String,
        user_id: String,
        email: Option<String>,
        firebase: TestFirebaseSignInContext,
        exp: usize,
    }

    #[derive(Serialize)]
    struct TestFirebaseSignInContext {
        sign_in_provider: String,
    }

    fn test_security_config() -> SecurityRuntimeConfig {
        SecurityRuntimeConfig {
            proxy_token: "proxy".to_string(),
            firebase_project_id: "demo-dastill".to_string(),
            allowed_origins: vec![],
            operator_email_allowlist: vec![],
            default_seeded_channel_id: "seeded".to_string(),
            baseline_rate_limit_per_minute: 60,
            expensive_rate_limit_per_minute: 10,
            anonymous_chat_quota: 10,
        }
    }

    #[test]
    fn build_verified_identity_maps_anonymous_sign_in_provider_to_anonymous_access() {
        let identity = build_verified_identity(super::FirebaseClaims {
            aud: "demo-dastill".to_string(),
            iss: "https://securetoken.google.com/demo-dastill".to_string(),
            sub: "firebase-anon".to_string(),
            user_id: Some("firebase-anon".to_string()),
            email: None,
            firebase: Some(super::FirebaseSignInContext {
                sign_in_provider: Some("anonymous".to_string()),
            }),
        });

        assert_eq!(
            identity,
            VerifiedClientIdentity {
                user_id: None,
                email: None,
                auth_state: FirebaseAuthState::Anonymous,
            }
        );
    }

    #[test]
    fn parse_cache_control_reads_max_age_directive() {
        assert_eq!(
            parse_cache_control_max_age("public, max-age=1800, must-revalidate"),
            Some(Duration::from_secs(1800))
        );
    }

    #[tokio::test]
    async fn verify_bearer_identity_accepts_emulator_tokens_without_signature_verification() {
        unsafe {
            std::env::set_var("FIREBASE_AUTH_EMULATOR_HOST", "127.0.0.1:9099");
        }

        let token = encode(
            &Header::default(),
            &TestClaims {
                aud: "demo-dastill".to_string(),
                iss: "https://securetoken.google.com/demo-dastill".to_string(),
                sub: "firebase-user-123".to_string(),
                user_id: "firebase-user-123".to_string(),
                email: Some("person@example.com".to_string()),
                firebase: TestFirebaseSignInContext {
                    sign_in_provider: "google.com".to_string(),
                },
                exp: 4_102_444_800,
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("encode test token");

        let identity = verify_bearer_identity(
            Some(&HeaderValue::from_str(&format!("Bearer {token}")).expect("header")),
            &test_security_config(),
        )
        .await
        .expect("verify token")
        .expect("identity");

        unsafe {
            std::env::remove_var("FIREBASE_AUTH_EMULATOR_HOST");
        }

        assert_eq!(
            identity,
            VerifiedClientIdentity {
                user_id: Some("firebase-user-123".to_string()),
                email: Some("person@example.com".to_string()),
                auth_state: FirebaseAuthState::Authenticated,
            }
        );
    }
}
