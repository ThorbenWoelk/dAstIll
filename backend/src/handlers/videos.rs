use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::db;
use crate::models::VideoInfo;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VideoTypeFilter {
    All,
    Long,
    Short,
}

impl VideoTypeFilter {
    fn as_is_short(self) -> Option<bool> {
        match self {
            Self::All => None,
            Self::Long => Some(false),
            Self::Short => Some(true),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct VideoListParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_shorts: Option<bool>,
    pub video_type: Option<VideoTypeFilter>,
    pub acknowledged: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct VideoInfoBackfillParams {
    pub limit: Option<usize>,
    pub force: Option<bool>,
}

pub async fn list_channel_videos(
    State(state): State<AppState>,
    Path(channel_id): Path<String>,
    Query(params): Query<VideoListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    {
        let conn = state.db.lock().await;
        let channel = db::get_channel(&conn, &channel_id)
            .await
            .map_err(map_db_err)?;
        if channel.is_none() {
            return Err((StatusCode::NOT_FOUND, "Channel not found".to_string()));
        }
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    tracing::info!("video_type filter: {:?}", params.video_type);
    let is_short = match params.video_type {
        Some(video_type) => video_type.as_is_short(),
        None => {
            let include_shorts = params.include_shorts.unwrap_or(true);
            if include_shorts { None } else { Some(false) }
        }
    };

    let conn = state.db.lock().await;
    let videos = db::list_videos_by_channel(
        &conn,
        &channel_id,
        limit,
        offset,
        is_short,
        params.acknowledged,
    )
    .await
    .map_err(map_db_err)?;

    Ok(Json(videos))
}

pub async fn get_video(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let video = db::get_video(&conn, &video_id).await.map_err(map_db_err)?;
    match video {
        Some(v) => Ok(Json(v)),
        None => Err((StatusCode::NOT_FOUND, "Video not found".to_string())),
    }
}

pub async fn get_video_info(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let video = {
        let conn = state.db.lock().await;
        db::get_video(&conn, &video_id).await.map_err(map_db_err)?
    };

    let video = match video {
        Some(video) => video,
        None => return Err((StatusCode::NOT_FOUND, "Video not found".to_string())),
    };

    {
        let conn = state.db.lock().await;
        if let Some(cached) = db::get_video_info(&conn, &video_id)
            .await
            .map_err(map_db_err)?
        {
            return Ok(Json(cached));
        }
    }

    match state.youtube.fetch_video_info(&video_id).await {
        Ok(mut info) => {
            if info.thumbnail_url.is_none() {
                info.thumbnail_url = video.thumbnail_url.clone();
            }
            if info.published_at.is_none() {
                info.published_at = Some(video.published_at);
            }
            if info.title.trim().is_empty() {
                info.title = video.title.clone();
            }
            let conn = state.db.lock().await;
            db::upsert_video_info(&conn, &info)
                .await
                .map_err(map_db_err)?;
            Ok(Json(info))
        }
        Err(err) => {
            tracing::warn!(
                video_id = %video_id,
                error = %err,
                "video info fetch failed - returning fallback metadata"
            );

            Ok(Json(VideoInfo {
                video_id: video_id.clone(),
                watch_url: format!("https://www.youtube.com/watch?v={video_id}"),
                title: video.title,
                description: None,
                thumbnail_url: video.thumbnail_url,
                channel_name: None,
                channel_id: Some(video.channel_id),
                published_at: Some(video.published_at),
                duration_iso8601: None,
                duration_seconds: None,
                view_count: None,
            }))
        }
    }
}

pub async fn backfill_video_info(
    State(state): State<AppState>,
    Query(params): Query<VideoInfoBackfillParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(100).clamp(1, 1000);
    let force = params.force.unwrap_or(false);

    let video_ids = {
        let conn = state.db.lock().await;
        if force {
            db::list_video_ids_for_info_refresh(&conn, limit)
                .await
                .map_err(map_db_err)?
        } else {
            db::list_video_ids_missing_info(&conn, limit)
                .await
                .map_err(map_db_err)?
        }
    };

    let mut updated = 0usize;
    let mut failed = 0usize;

    for video_id in &video_ids {
        let mut info = match state.youtube.fetch_video_info(video_id).await {
            Ok(info) => info,
            Err(err) => {
                failed += 1;
                tracing::warn!(
                    video_id = %video_id,
                    error = %err,
                    "video info backfill fetch failed"
                );
                continue;
            }
        };

        let video = {
            let conn = state.db.lock().await;
            db::get_video(&conn, video_id).await.map_err(map_db_err)?
        };

        if let Some(video) = video {
            if info.thumbnail_url.is_none() {
                info.thumbnail_url = video.thumbnail_url;
            }
            if info.published_at.is_none() {
                info.published_at = Some(video.published_at);
            }
            if info.title.trim().is_empty() {
                info.title = video.title;
            }
        }

        let conn = state.db.lock().await;
        match db::upsert_video_info(&conn, &info).await {
            Ok(_) => updated += 1,
            Err(err) => {
                failed += 1;
                tracing::warn!(
                    video_id = %video_id,
                    error = %err,
                    "video info backfill store failed"
                );
            }
        }
    }

    Ok(Json(serde_json::json!({
        "requested_limit": limit,
        "force": force,
        "processed": video_ids.len(),
        "updated": updated,
        "failed": failed
    })))
}

pub async fn update_video_acknowledged(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
    Json(payload): Json<crate::models::UpdateAcknowledgedRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let video = db::get_video(&conn, &video_id).await.map_err(map_db_err)?;
    match video {
        Some(mut v) => {
            db::update_video_acknowledged(&conn, &video_id, payload.acknowledged)
                .await
                .map_err(map_db_err)?;
            v.acknowledged = payload.acknowledged;
            Ok(Json(v))
        }
        None => Err((StatusCode::NOT_FOUND, "Video not found".to_string())),
    }
}

fn map_db_err(err: libsql::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
