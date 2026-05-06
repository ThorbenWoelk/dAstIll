use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    audit, db,
    models::{CreateHighlightRequest, Highlight, HighlightChannelGroup},
    security::{AccessContext, AuthState},
    state::AppState,
};

use super::{map_db_err, require_video_for_access, validate_nonempty};

#[utoipa::path(
    post,
    path = "/api/videos/{id}/highlights",
    params(
        ("id" = String, Path, description = "Video id")
    ),
    request_body = CreateHighlightRequest,
    responses(
        (status = 201, description = "Created highlight", body = Highlight),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Video not found", body = String),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn create_highlight(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
    Json(payload): Json<CreateHighlightRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    require_video_for_access(&state, &access_context, &video_id).await?;
    let highlight_text = validate_nonempty(&payload.text, "Highlight text cannot be empty")?;

    let highlight = db::create_highlight(
        &state.db,
        user_id,
        &video_id,
        payload.source,
        highlight_text,
        &payload.prefix_context,
        &payload.suffix_context,
    )
    .await
    .map_err(map_db_err)?;

    audit::log_highlight_create(user_id, &video_id, highlight.id, highlight.source);

    Ok((StatusCode::CREATED, Json(highlight)))
}

#[utoipa::path(
    get,
    path = "/api/videos/{id}/highlights",
    params(
        ("id" = String, Path, description = "Video id")
    ),
    responses(
        (status = 200, description = "Highlights for the video", body = [Highlight]),
        (status = 404, description = "Video not found", body = String),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn list_video_highlights(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Ok(Json(Vec::new()));
    };
    require_video_for_access(&state, &access_context, &video_id).await?;

    let highlights = db::list_video_highlights(&state.db, user_id, &video_id)
        .await
        .map_err(map_db_err)?;
    Ok(Json(highlights))
}

#[utoipa::path(
    get,
    path = "/api/highlights",
    responses(
        (status = 200, description = "Highlights grouped by source and video", body = [HighlightChannelGroup]),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn list_highlights(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Ok(Json(Vec::new()));
    };
    let grouped: Vec<HighlightChannelGroup> =
        db::list_highlights_grouped_for_user(&state.db, user_id)
            .await
            .map_err(map_db_err)?;
    Ok(Json(grouped))
}

fn resolve_delete_highlight_result(deleted: bool) -> Result<StatusCode, (StatusCode, String)> {
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Highlight not found".to_string()))
    }
}

#[utoipa::path(
    delete,
    path = "/api/highlights/{id}",
    params(
        ("id" = i64, Path, description = "Highlight id")
    ),
    responses(
        (status = 204, description = "Deleted highlight"),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Highlight not found", body = String)
    )
)]
pub async fn delete_highlight(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(highlight_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    let deleted = db::delete_highlight(&state.db, user_id, highlight_id)
        .await
        .map_err(map_db_err)?;
    if deleted {
        audit::log_highlight_delete(user_id, highlight_id);
    }
    let status = resolve_delete_highlight_result(deleted)?;

    Ok(status)
}

#[cfg(test)]
#[path = "highlights_tests.rs"]
mod highlights_tests;
