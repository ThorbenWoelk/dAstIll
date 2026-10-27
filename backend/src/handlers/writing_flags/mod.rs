use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    audit, db,
    models::{CreateWritingFlagRequest, WritingFlag},
    security::{AccessContext, AuthState},
    state::AppState,
};

use super::{map_db_err, require_video_for_access, validate_nonempty};

/// Longest flagged passage accepted, in characters.
const MAX_FLAG_TEXT_CHARS: usize = 2_000;
/// Longest note accepted, in characters.
const MAX_FLAG_COMMENT_CHARS: usize = 1_000;

fn require_signed_in(access_context: &AccessContext) -> Result<&str, (StatusCode, String)> {
    match access_context.user_id.as_deref() {
        Some(user_id) if access_context.auth_state == AuthState::Authenticated => Ok(user_id),
        _ => Err((StatusCode::FORBIDDEN, "Sign-in required".to_string())),
    }
}

/// The flagged passage and the reader's note, trimmed and within limits.
fn validate_writing_flag(
    payload: &CreateWritingFlagRequest,
) -> Result<(&str, &str), (StatusCode, String)> {
    let text = validate_nonempty(&payload.text, "Flagged text cannot be empty")?;
    let comment = validate_nonempty(&payload.comment, "Say what is wrong with the passage")?;
    if text.chars().count() > MAX_FLAG_TEXT_CHARS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Flag at most {MAX_FLAG_TEXT_CHARS} characters"),
        ));
    }
    if comment.chars().count() > MAX_FLAG_COMMENT_CHARS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Keep the note under {MAX_FLAG_COMMENT_CHARS} characters"),
        ));
    }
    Ok((text, comment))
}

#[utoipa::path(
    post,
    path = "/api/videos/{id}/writing-flags",
    params(
        ("id" = String, Path, description = "Video id")
    ),
    request_body = CreateWritingFlagRequest,
    responses(
        (status = 201, description = "Flagged passage", body = WritingFlag),
        (status = 400, description = "Missing or too long text or note", body = String),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Video not found", body = String),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn create_writing_flag(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
    Json(payload): Json<CreateWritingFlagRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_signed_in(&access_context)?;
    require_video_for_access(&state, &access_context, &video_id).await?;
    let (text, comment) = validate_writing_flag(&payload)?;

    let flag = db::create_writing_flag(
        &state.db,
        user_id,
        &video_id,
        text,
        comment,
        &payload.prefix_context,
        &payload.suffix_context,
    )
    .await
    .map_err(map_db_err)?;

    audit::log_writing_flag_create(user_id, &video_id, flag.id);

    Ok((StatusCode::CREATED, Json(flag)))
}

#[utoipa::path(
    get,
    path = "/api/videos/{id}/writing-flags",
    params(
        ("id" = String, Path, description = "Video id")
    ),
    responses(
        (status = 200, description = "The reader's flags on this video", body = [WritingFlag]),
        (status = 404, description = "Video not found", body = String),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn list_video_writing_flags(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Ok(user_id) = require_signed_in(&access_context) else {
        return Ok(Json(Vec::new()));
    };
    require_video_for_access(&state, &access_context, &video_id).await?;

    let flags = db::list_video_writing_flags(&state.db, user_id, &video_id)
        .await
        .map_err(map_db_err)?;
    Ok(Json(flags))
}

#[utoipa::path(
    get,
    path = "/api/writing-flags",
    responses(
        (status = 200, description = "The reader's flags, newest first", body = [WritingFlag]),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn list_writing_flags(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Ok(user_id) = require_signed_in(&access_context) else {
        return Ok(Json(Vec::new()));
    };
    let flags = db::list_user_writing_flags(&state.db, user_id)
        .await
        .map_err(map_db_err)?;
    Ok(Json(flags))
}

#[utoipa::path(
    delete,
    path = "/api/writing-flags/{id}",
    params(
        ("id" = i64, Path, description = "Writing flag id")
    ),
    responses(
        (status = 204, description = "Removed the flag"),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Flag not found", body = String)
    )
)]
pub async fn delete_writing_flag(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(flag_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_signed_in(&access_context)?;
    let deleted = db::delete_writing_flag(&state.db, user_id, flag_id)
        .await
        .map_err(map_db_err)?;
    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Flag not found".to_string()));
    }
    audit::log_writing_flag_delete(user_id, flag_id);
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
