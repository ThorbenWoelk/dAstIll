use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    db,
    models::UserPreferences,
    security::{AccessContext, AuthState},
    state::AppState,
};

use super::map_db_err;

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

    db::save_user_preferences(&state.db, user_id, &payload)
        .await
        .map_err(map_db_err)?;
    Ok(StatusCode::NO_CONTENT)
}
