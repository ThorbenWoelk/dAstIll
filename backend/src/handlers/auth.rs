use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{
    security::{AccessContext, AuthState, CLIENT_IP_HEADER},
    state::{AppState, MobileAuthHandoff},
};

const MOBILE_AUTH_HANDOFF_TTL: Duration = Duration::from_secs(5 * 60);
const MOBILE_AUTH_HANDOFF_TOKEN_BYTES: usize = 32;
const X_FORWARDED_FOR_HEADER: &str = "x-forwarded-for";

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMobileAuthHandoffPayload {
    pub status: String,
    pub handoff_id: String,
    pub complete_token: String,
    pub redeem_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MobileAuthHandoffStatusPayload {
    pub status: String,
    pub google_id_token: Option<String>,
    pub google_access_token: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteMobileAuthHandoffPayload {
    pub complete_token: String,
    pub google_id_token: String,
    pub google_access_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RedeemMobileAuthHandoffPayload {
    pub redeem_token: String,
}

fn prune_expired_handoffs(state: &mut std::collections::HashMap<String, MobileAuthHandoff>) {
    let now = Instant::now();
    state.retain(|_, handoff| now.duration_since(handoff.created_at) < MOBILE_AUTH_HANDOFF_TTL);
}

fn mint_nonce() -> String {
    let mut bytes = [0_u8; MOBILE_AUTH_HANDOFF_TOKEN_BYTES];
    rustls::crypto::ring::default_provider()
        .secure_random
        .fill(&mut bytes)
        .expect("OS randomness must be available for auth handoff secrets");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_value(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn token_matches(expected_hash: &str, candidate: &str) -> bool {
    hash_value(candidate) == expected_hash
}

fn handoff_not_found() -> (StatusCode, String) {
    (
        StatusCode::NOT_FOUND,
        "Mobile auth handoff not found".to_string(),
    )
}

fn handoff_pending_payload() -> MobileAuthHandoffStatusPayload {
    MobileAuthHandoffStatusPayload {
        status: "pending".to_string(),
        google_id_token: None,
        google_access_token: None,
    }
}

fn extract_client_hint(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(CLIENT_IP_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            headers
                .get(X_FORWARDED_FOR_HEADER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(',').next())
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
}

fn resolve_creator_binding(headers: &HeaderMap, access_context: &AccessContext) -> String {
    if let Some(authorization) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("authorization:{}", hash_value(authorization));
    }

    if access_context.auth_state == AuthState::Authenticated {
        if let Some(user_id) = access_context.user_id.as_deref() {
            return format!("user:{user_id}");
        }
    }

    if let Some(client_hint) = extract_client_hint(headers) {
        return format!("client:{}", hash_value(client_hint));
    }

    format!("scope:{}", access_context.cache_scope_key())
}

#[utoipa::path(
    post,
    path = "/api/auth/mobile-handoff",
    responses(
        (status = 201, description = "Created a mobile auth handoff session", body = CreateMobileAuthHandoffPayload)
    )
)]
pub async fn create_mobile_auth_handoff(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let handoff_id = mint_nonce();
    let complete_token = mint_nonce();
    let redeem_token = mint_nonce();
    let creator_binding = resolve_creator_binding(&headers, &access_context);

    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    handoffs.insert(
        handoff_id.clone(),
        MobileAuthHandoff {
            created_at: Instant::now(),
            creator_binding_hash: hash_value(&creator_binding),
            complete_token_hash: hash_value(&complete_token),
            redeem_token_hash: hash_value(&redeem_token),
            google_id_token: None,
            google_access_token: None,
        },
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateMobileAuthHandoffPayload {
            status: "pending".to_string(),
            handoff_id,
            complete_token,
            redeem_token,
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/api/auth/mobile-handoff/{id}",
    params(
        ("id" = String, Path, description = "Mobile auth handoff session id")
    ),
    request_body = CompleteMobileAuthHandoffPayload,
    responses(
        (status = 200, description = "Completed handoff session", body = MobileAuthHandoffStatusPayload),
        (status = 400, description = "Request failed", body = String),
        (status = 403, description = "Authenticated handoff completion required", body = String),
        (status = 404, description = "Unknown handoff session", body = String),
        (status = 409, description = "Handoff session already completed", body = String)
    )
)]
pub async fn complete_mobile_auth_handoff(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(session_id): Path<String>,
    Json(payload): Json<CompleteMobileAuthHandoffPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if access_context.auth_state != AuthState::Authenticated {
        return Err((
            StatusCode::FORBIDDEN,
            "Authenticated handoff completion required".to_string(),
        ));
    }

    let complete_token = payload.complete_token.trim();
    let google_id_token = payload.google_id_token.trim();
    let google_access_token = payload.google_access_token.trim();
    if complete_token.is_empty() || google_id_token.is_empty() || google_access_token.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Complete token and Google auth tokens are required".to_string(),
        ));
    }

    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    let Some(handoff) = handoffs.get_mut(&session_id) else {
        return Err(handoff_not_found());
    };

    if !token_matches(&handoff.complete_token_hash, complete_token) {
        return Err(handoff_not_found());
    }

    if handoff.google_id_token.is_some() || handoff.google_access_token.is_some() {
        return Err((
            StatusCode::CONFLICT,
            "Mobile auth handoff already completed".to_string(),
        ));
    }

    handoff.google_id_token = Some(google_id_token.to_string());
    handoff.google_access_token = Some(google_access_token.to_string());

    Ok(Json(MobileAuthHandoffStatusPayload {
        status: "complete".to_string(),
        google_id_token: None,
        google_access_token: None,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/mobile-handoff/{id}/redeem",
    params(
        ("id" = String, Path, description = "Mobile auth handoff session id")
    ),
    request_body = RedeemMobileAuthHandoffPayload,
    responses(
        (status = 200, description = "Redeemed completed handoff session", body = MobileAuthHandoffStatusPayload),
        (status = 202, description = "Handoff session exists but browser login is still pending", body = MobileAuthHandoffStatusPayload),
        (status = 400, description = "Request failed", body = String),
        (status = 404, description = "Unknown handoff session", body = String)
    )
)]
pub async fn redeem_mobile_auth_handoff(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(payload): Json<RedeemMobileAuthHandoffPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let redeem_token = payload.redeem_token.trim();
    if redeem_token.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Redeem token is required".to_string(),
        ));
    }

    let creator_binding = resolve_creator_binding(&headers, &access_context);
    let creator_binding_hash = hash_value(&creator_binding);

    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    let Some(handoff) = handoffs.get(&session_id) else {
        return Err(handoff_not_found());
    };

    if handoff.creator_binding_hash != creator_binding_hash
        || !token_matches(&handoff.redeem_token_hash, redeem_token)
    {
        return Err(handoff_not_found());
    }

    if handoff.google_id_token.is_none() || handoff.google_access_token.is_none() {
        return Ok((StatusCode::ACCEPTED, Json(handoff_pending_payload())));
    }

    let handoff = handoffs.remove(&session_id).ok_or_else(handoff_not_found)?;

    Ok((
        StatusCode::OK,
        Json(MobileAuthHandoffStatusPayload {
            status: "complete".to_string(),
            google_id_token: handoff.google_id_token,
            google_access_token: handoff.google_access_token,
        }),
    ))
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod auth_tests;
