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
        default_seeded_channel_ids: vec!["seeded".to_string()],
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
