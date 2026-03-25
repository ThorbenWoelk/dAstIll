use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{db, models::UserPreferences, state::AppState};

use super::map_db_err;

pub async fn get_preferences(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let prefs = db::get_preferences(&state.db).await.map_err(map_db_err)?;
    Ok(Json(prefs))
}

pub async fn save_preferences(
    State(state): State<AppState>,
    Json(payload): Json<UserPreferences>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    db::save_preferences(&state.db, &payload)
        .await
        .map_err(map_db_err)?;
    Ok(StatusCode::NO_CONTENT)
}
