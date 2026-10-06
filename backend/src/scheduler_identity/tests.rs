use axum::http::HeaderValue;

use super::{SchedulerClaims, check_scheduler_identity, verify_scheduler_request};
use crate::config::CatchUpRuntimeConfig;

fn config() -> CatchUpRuntimeConfig {
    CatchUpRuntimeConfig {
        invoker_email: "dastill-scheduler-sa@dastill.iam.gserviceaccount.com".to_string(),
        audience: "dastill-catch-up".to_string(),
    }
}

fn claims(email: Option<&str>, email_verified: Option<bool>) -> SchedulerClaims {
    SchedulerClaims {
        email: email.map(str::to_string),
        email_verified,
    }
}

#[test]
fn accepts_the_configured_verified_service_account() {
    let claims = claims(
        Some("DAstill-scheduler-sa@dastill.iam.gserviceaccount.com"),
        Some(true),
    );
    assert!(check_scheduler_identity(&claims, &config()).is_ok());
}

#[test]
fn rejects_other_accounts_and_unverified_emails() {
    let other = claims(Some("someone@example.com"), Some(true));
    assert!(check_scheduler_identity(&other, &config()).is_err());

    let unverified = claims(
        Some("dastill-scheduler-sa@dastill.iam.gserviceaccount.com"),
        Some(false),
    );
    assert!(check_scheduler_identity(&unverified, &config()).is_err());

    let missing = claims(None, None);
    assert!(check_scheduler_identity(&missing, &config()).is_err());
}

#[tokio::test]
async fn rejects_requests_without_a_bearer_token() {
    let error = verify_scheduler_request(None, &config())
        .await
        .expect_err("missing token must be rejected");
    assert!(error.contains("missing"), "{error}");
}

#[tokio::test]
async fn rejects_tokens_that_are_not_jwts() {
    let header = HeaderValue::from_static("Bearer not-a-jwt");
    assert!(
        verify_scheduler_request(Some(&header), &config())
            .await
            .is_err()
    );
}
