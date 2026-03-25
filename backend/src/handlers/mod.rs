pub mod analytics;
pub mod channels;
pub mod chat;
pub mod content;
pub mod highlights;
pub mod preferences;
pub mod query;
pub mod search;
pub mod videos;

use axum::http::StatusCode;

use crate::{
    db,
    models::{Channel, Video},
    state::AppState,
};

pub(crate) fn map_db_err(err: crate::db::StoreError) -> (axum::http::StatusCode, String) {
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
    db::get_channel(&state.db, channel_id)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Channel not found".to_string()))
}

pub(crate) async fn require_video(
    state: &AppState,
    video_id: &str,
) -> Result<Video, (StatusCode, String)> {
    db::get_video(&state.db, video_id, true)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Video not found".to_string()))
}

pub(crate) async fn evict_video_scope_cache(
    state: &AppState,
    channel_id: &str,
) -> Result<(), (StatusCode, String)> {
    let is_subscribed = db::get_channel(&state.db, channel_id)
        .await
        .map_err(map_db_err)?
        .is_some();

    if is_subscribed {
        state.read_cache.evict_channel(channel_id).await;
    } else {
        state
            .read_cache
            .evict_channel(crate::models::OTHERS_CHANNEL_ID)
            .await;
    }

    Ok(())
}
