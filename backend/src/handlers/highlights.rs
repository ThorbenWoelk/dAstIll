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

use super::{map_db_err, require_video, validate_nonempty};

pub async fn create_highlight(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Json(payload): Json<CreateHighlightRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_video(&state, &video_id).await?;
    let highlight_text = validate_nonempty(&payload.text, "Highlight text cannot be empty")?;

    let highlight = db::create_highlight(
        &state.db,
        &video_id,
        payload.source,
        highlight_text,
        &payload.prefix_context,
        &payload.suffix_context,
    )
    .await
    .map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(highlight)))
}

pub async fn list_video_highlights(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_video(&state, &video_id).await?;

    let highlights = db::list_video_highlights(&state.db, &video_id)
        .await
        .map_err(map_db_err)?;
    Ok(Json(highlights))
}

pub async fn list_highlights(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let grouped: Vec<HighlightChannelGroup> = db::list_highlights_grouped(&state.db)
        .await
        .map_err(map_db_err)?;
    Ok(Json(grouped))
}

pub async fn delete_highlight(
    State(state): State<AppState>,
    Path(highlight_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let deleted = db::delete_highlight(&state.db, highlight_id)
        .await
        .map_err(map_db_err)?;
    let status = resolve_delete_highlight_result(deleted)?;

    Ok(status)
}

fn resolve_delete_highlight_result(deleted: bool) -> Result<StatusCode, (StatusCode, String)> {
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Highlight not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::resolve_delete_highlight_result;

    #[test]
    fn delete_highlight_result_maps_missing_rows_to_not_found() {
        assert_eq!(
            resolve_delete_highlight_result(true).unwrap(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            resolve_delete_highlight_result(false).unwrap_err().0,
            StatusCode::NOT_FOUND
        );
    }
}
