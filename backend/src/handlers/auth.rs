use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::state::{AppState, MobileAuthHandoff};

const MOBILE_AUTH_HANDOFF_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Serialize)]
pub struct MobileAuthHandoffStatusPayload {
    pub status: &'static str,
    pub google_id_token: Option<String>,
    pub google_access_token: Option<String>,
}

#[derive(Deserialize)]
pub struct CompleteMobileAuthHandoffPayload {
    pub google_id_token: String,
    pub google_access_token: String,
}

fn prune_expired_handoffs(state: &mut std::collections::HashMap<String, MobileAuthHandoff>) {
    let now = Instant::now();
    state.retain(|_, handoff| now.duration_since(handoff.created_at) < MOBILE_AUTH_HANDOFF_TTL);
}

pub async fn create_mobile_auth_handoff(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    handoffs
        .entry(session_id)
        .or_insert_with(|| MobileAuthHandoff {
            created_at: Instant::now(),
            google_id_token: None,
            google_access_token: None,
        });

    Ok((StatusCode::CREATED, Json(MobileAuthHandoffStatusPayload {
        status: "pending",
        google_id_token: None,
        google_access_token: None,
    })))
}

pub async fn complete_mobile_auth_handoff(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(payload): Json<CompleteMobileAuthHandoffPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let google_id_token = payload.google_id_token.trim();
    let google_access_token = payload.google_access_token.trim();
    if google_id_token.is_empty() || google_access_token.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Google auth tokens are required".to_string()));
    }

    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    let handoff = handoffs
        .entry(session_id)
        .or_insert_with(|| MobileAuthHandoff {
            created_at: Instant::now(),
            google_id_token: None,
            google_access_token: None,
        });
    handoff.google_id_token = Some(google_id_token.to_string());
    handoff.google_access_token = Some(google_access_token.to_string());

    Ok(Json(MobileAuthHandoffStatusPayload {
        status: "complete",
        google_id_token: None,
        google_access_token: None,
    }))
}

pub async fn get_mobile_auth_handoff(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    prune_expired_handoffs(&mut handoffs);
    let Some(handoff) = handoffs.get(&session_id) else {
        return Ok(Json(MobileAuthHandoffStatusPayload {
            status: "pending",
            google_id_token: None,
            google_access_token: None,
        }));
    };

    if let (Some(id_token), Some(access_token)) = (
        handoff.google_id_token.clone(),
        handoff.google_access_token.clone(),
    ) {
        return Ok(Json(MobileAuthHandoffStatusPayload {
            status: "complete",
            google_id_token: Some(id_token),
            google_access_token: Some(access_token),
        }));
    }

    Ok(Json(MobileAuthHandoffStatusPayload {
        status: "pending",
        google_id_token: None,
        google_access_token: None,
    }))
}

pub async fn delete_mobile_auth_handoff(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut handoffs = state.mobile_auth_handoffs.lock().await;
    handoffs.remove(&session_id);
    Ok(StatusCode::NO_CONTENT)
}
