use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    db,
    models::{CreateHighlightRequest, HighlightChannelGroup},
    state::AppState,
};

use super::{map_db_err, require_video};

pub async fn create_highlight(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Json(payload): Json<CreateHighlightRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_video(&state, &video_id).await?;

    if payload.text.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Highlight text cannot be empty".to_string(),
        ));
    }

    let conn = state.db.connect();
    let highlight = db::create_highlight(
        &conn,
        &video_id,
        payload.source,
        payload.text.trim(),
        &payload.prefix_context,
        &payload.suffix_context,
    )
    .await
    .map_err(map_db_err)?;
    state.read_cache.clear().await;

    Ok((StatusCode::CREATED, Json(highlight)))
}

pub async fn list_video_highlights(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_video(&state, &video_id).await?;

    let conn = state.db.connect();
    let highlights = db::list_video_highlights(&conn, &video_id)
        .await
        .map_err(map_db_err)?;
    Ok(Json(highlights))
}

pub async fn list_highlights(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.connect();
    let grouped: Vec<HighlightChannelGroup> = db::list_highlights_grouped(&conn)
        .await
        .map_err(map_db_err)?;
    Ok(Json(grouped))
}

pub async fn delete_highlight(
    State(state): State<AppState>,
    Path(highlight_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.connect();
    let deleted = db::delete_highlight(&conn, highlight_id)
        .await
        .map_err(map_db_err)?;

    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Highlight not found".to_string()));
    }

    state.read_cache.clear().await;
    Ok(StatusCode::NO_CONTENT)
}
