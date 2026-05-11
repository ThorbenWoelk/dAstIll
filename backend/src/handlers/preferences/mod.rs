use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    audit, db,
    models::UserPreferences,
    security::{AccessContext, AuthState},
    state::AppState,
};

use super::map_db_err;

#[utoipa::path(
    get,
    path = "/api/preferences",
    responses(
        (status = 200, description = "User preferences", body = UserPreferences),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn get_preferences(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let prefs = match access_context.user_id.as_deref() {
        Some(user_id) if access_context.auth_state == AuthState::Authenticated => {
            db::get_user_preferences(&state.db, user_id)
                .await
                .map_err(map_db_err)?
        }
        _ => UserPreferences::default(),
    };
    Ok(Json(prefs))
}

#[utoipa::path(
    put,
    path = "/api/preferences",
    request_body = UserPreferences,
    responses(
        (status = 204, description = "Saved preferences"),
        (status = 403, description = "Sign-in required", body = String),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn save_preferences(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<UserPreferences>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    let before = db::get_user_preferences(&state.db, user_id)
        .await
        .map_err(map_db_err)?;
    db::save_user_preferences(&state.db, user_id, &payload)
        .await
        .map_err(map_db_err)?;
    audit::log_preferences_save(user_id, &before, &payload);
    Ok(StatusCode::NO_CONTENT)
}
