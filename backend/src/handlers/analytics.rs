use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::Value;

use crate::state::AppState;

const MAX_BATCH_SIZE: usize = 200;

#[utoipa::path(
    post,
    path = "/api/analytics/events",
    request_body(
        content = String,
        content_type = "application/json",
        description = "JSON array of analytics event payloads"
    ),
    responses(
        (status = 202, description = "Accepted analytics batch"),
        (status = 204, description = "Ignored empty batch or disabled analytics sink"),
        (status = 413, description = "Batch exceeded the maximum size", body = String)
    )
)]
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
        return Ok(StatusCode::NO_CONTENT);
    };

    if let Err(error) = analytics.enqueue_events(events) {
        tracing::warn!(error = %error, "analytics batch dropped before enqueue");
    }

    Ok(StatusCode::ACCEPTED)
}
