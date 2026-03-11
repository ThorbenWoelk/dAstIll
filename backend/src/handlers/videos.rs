use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::db;
use crate::models::{Video, VideoInfo};
use crate::state::AppState;

pub fn resolve_is_short(
    video_type: Option<VideoTypeFilter>,
    include_shorts: Option<bool>,
) -> Option<bool> {
    match video_type {
        Some(vt) => vt.as_is_short(),
        None => {
            let include = include_shorts.unwrap_or(true);
            if include { None } else { Some(false) }
        }
    }
}

pub fn resolve_queue_filter(
    queue_tab: Option<QueueTab>,
    queue_only: Option<bool>,
) -> Option<db::QueueFilter> {
    match queue_tab {
        Some(QueueTab::Transcripts) => Some(db::QueueFilter::TranscriptsOnly),
        Some(QueueTab::Summaries) => Some(db::QueueFilter::SummariesOnly),
        Some(QueueTab::Evaluations) => Some(db::QueueFilter::EvaluationsOnly),
        None if queue_only.unwrap_or(false) => Some(db::QueueFilter::AnyIncomplete),
        None => None,
    }
}

pub fn enrich_video_info(info: &mut VideoInfo, video: &Video) {
    if info.thumbnail_url.is_none() {
        info.thumbnail_url = video.thumbnail_url.clone();
    }
    if info.published_at.is_none() {
        info.published_at = Some(video.published_at);
    }
    if info.title.trim().is_empty() {
        info.title = video.title.clone();
    }
}

fn cached_video_info_needs_refresh(info: &VideoInfo) -> bool {
    info.duration_seconds.is_none()
        && info
            .duration_iso8601
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VideoTypeFilter {
    All,
    Long,
    Short,
}

impl VideoTypeFilter {
    pub fn as_is_short(self) -> Option<bool> {
        match self {
            Self::All => None,
            Self::Long => Some(false),
            Self::Short => Some(true),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueueTab {
    Transcripts,
    Summaries,
    Evaluations,
}

#[derive(Debug, Deserialize)]
pub struct VideoListParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_shorts: Option<bool>,
    pub video_type: Option<VideoTypeFilter>,
    pub acknowledged: Option<bool>,
    pub queue_only: Option<bool>,
    pub queue_tab: Option<QueueTab>,
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
    let is_short = resolve_is_short(params.video_type, params.include_shorts);
    let queue_filter = resolve_queue_filter(params.queue_tab, params.queue_only);
    let conn = state.db.lock().await;
    let videos = db::list_videos_by_channel(
        &conn,
        &channel_id,
        limit,
        offset,
        is_short,
        params.acknowledged,
        queue_filter,
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

    let cached = {
        let conn = state.db.lock().await;
        db::get_video_info(&conn, &video_id)
            .await
            .map_err(map_db_err)?
    };

    if let Some(cached) = cached.as_ref() {
        if !cached_video_info_needs_refresh(cached) {
            return Ok(Json(cached.clone()));
        }
    }

    match state.youtube.fetch_video_info(&video_id).await {
        Ok(mut info) => {
            enrich_video_info(&mut info, &video);
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
                "video info fetch failed - returning cached or fallback metadata"
            );

            if let Some(cached) = cached {
                return Ok(Json(cached));
            }

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
            enrich_video_info(&mut info, &video);
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

use super::map_db_err;

#[cfg(test)]
mod tests {
    use super::cached_video_info_needs_refresh;
    use crate::models::VideoInfo;

    fn build_video_info(
        duration_seconds: Option<u64>,
        duration_iso8601: Option<&str>,
    ) -> VideoInfo {
        VideoInfo {
            video_id: "video-123".to_string(),
            watch_url: "https://www.youtube.com/watch?v=video-123".to_string(),
            title: "Video".to_string(),
            description: None,
            thumbnail_url: None,
            channel_name: None,
            channel_id: None,
            published_at: None,
            duration_iso8601: duration_iso8601.map(str::to_string),
            duration_seconds,
            view_count: None,
        }
    }

    #[test]
    fn cached_video_info_needs_refresh_when_duration_is_missing() {
        assert!(cached_video_info_needs_refresh(&build_video_info(
            None, None
        )));
        assert!(cached_video_info_needs_refresh(&build_video_info(
            None,
            Some(""),
        )));
    }

    #[test]
    fn cached_video_info_with_known_duration_does_not_need_refresh() {
        assert!(!cached_video_info_needs_refresh(&build_video_info(
            Some(185),
            None,
        )));
        assert!(!cached_video_info_needs_refresh(&build_video_info(
            None,
            Some("PT3M5S"),
        )));
    }
}
