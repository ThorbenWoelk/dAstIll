pub mod channels;
pub mod content;
pub mod query;
pub mod videos;

use axum::http::StatusCode;

use crate::{
    db,
    models::{Channel, Video},
    state::AppState,
};

pub(crate) fn map_db_err(err: libsql::Error) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        err.to_string(),
    )
}

pub(crate) fn map_internal_err(err: impl std::fmt::Display) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        err.to_string(),
    )
}

pub(crate) async fn require_channel(
    state: &AppState,
    channel_id: &str,
) -> Result<Channel, (StatusCode, String)> {
    let conn = state.db.connect();
    db::get_channel(&conn, channel_id)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Channel not found".to_string()))
}

pub(crate) async fn require_video(
    state: &AppState,
    video_id: &str,
) -> Result<Video, (StatusCode, String)> {
    let conn = state.db.connect();
    db::get_video(&conn, video_id)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Video not found".to_string()))
}
