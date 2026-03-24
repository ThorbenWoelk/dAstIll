use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::Value;

use crate::state::AppState;

const MAX_BATCH_SIZE: usize = 200;

pub async fn ingest_events(
    State(state): State<AppState>,
    Json(events): Json<Vec<Value>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if events.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    if events.len() > MAX_BATCH_SIZE {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("batch exceeds maximum of {MAX_BATCH_SIZE} events"),
        ));
    }

    let Some(analytics) = state.analytics.as_ref() else {
        tracing::warn!("analytics sink is not configured; dropping batch");
        return Ok(StatusCode::NO_CONTENT);
    };

    if let Err(error) = analytics.enqueue_events(events) {
        tracing::warn!(error = %error, "analytics batch dropped before enqueue");
    }

    Ok(StatusCode::ACCEPTED)
}
