use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::db;
use crate::models::{Video, VideoInfo};
use crate::state::AppState;

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
    if info.duration_seconds.is_none()
        && info
            .duration_iso8601
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return true;
    }
    info.description.as_deref().map_or(false, |d| {
        crate::services::youtube::placeholder::is_site_wide_placeholder_description(d)
    })
}
use super::query::VideoListParams;
use super::{map_db_err, require_channel, require_video};

#[derive(Debug, serde::Deserialize)]
pub struct VideoInfoBackfillParams {
    pub limit: Option<usize>,
    pub force: Option<bool>,
    /// Re-fetch stored video info rows whose description is a known YouTube site-wide blurb.
    pub heal_placeholders: Option<bool>,
}

pub async fn list_channel_videos(
    State(state): State<AppState>,
    Path(channel_id): Path<String>,
    Query(params): Query<VideoListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_channel(&state, &channel_id).await?;

    tracing::info!("video_type filter: {:?}", params.video_type);
    let conn = state.db.connect();
    let videos = db::list_videos_by_channel(
        &conn,
        &channel_id,
        params.limit_or_default(),
        params.offset_or_default(),
        params.is_short_filter(),
        params.acknowledged_filter(),
        params.queue_filter(),
    )
    .await
    .map_err(map_db_err)?;

    Ok(Json(videos))
}

pub async fn get_video(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(require_video(&state, &video_id).await?))
}

pub async fn get_video_info(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let video = require_video(&state, &video_id).await?;

    let cached = {
        let conn = state.db.connect();
        db::get_video_info(&conn, &video_id)
            .await
            .map_err(map_db_err)?
    };

    let mut cached = cached.ok_or((StatusCode::NOT_FOUND, "Video info not found".to_string()))?;
    enrich_video_info(&mut cached, &video);
    Ok(Json(cached))
}

pub async fn ensure_video_info(
    State(state): State<AppState>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let video = require_video(&state, &video_id).await?;

    let cached = {
        let conn = state.db.connect();
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
            let conn = state.db.connect();
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

    let heal = params.heal_placeholders.unwrap_or(false);
    let video_ids = {
        let conn = state.db.connect();
        if heal {
            db::list_video_ids_with_placeholder_descriptions(&conn, limit)
                .await
                .map_err(map_db_err)?
        } else if force {
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
            let conn = state.db.connect();
            db::get_video(&conn, video_id, false)
                .await
                .map_err(map_db_err)?
        };

        if let Some(video) = video {
            enrich_video_info(&mut info, &video);
        }

        let conn = state.db.connect();
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
        "heal_placeholders": heal,
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
    let mut video = require_video(&state, &video_id).await?;
    let conn = state.db.connect();
    db::update_video_acknowledged(&conn, &video_id, payload.acknowledged)
        .await
        .map_err(map_db_err)?;
    video.acknowledged = payload.acknowledged;
    state.read_cache.evict_channel(&video.channel_id).await;
    Ok(Json(video))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{VideoListParams, cached_video_info_needs_refresh, enrich_video_info};
    use crate::db;
    use crate::handlers::query::{QueueTab, VideoTypeFilter};
    use crate::models::{ContentStatus, Video, VideoInfo};

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

    fn build_video() -> Video {
        Video {
            id: "video-123".to_string(),
            channel_id: "channel-123".to_string(),
            title: "Stored Video Title".to_string(),
            thumbnail_url: Some("https://example.com/thumb.jpg".to_string()),
            published_at: Utc::now(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
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

    #[test]
    fn cached_video_info_needs_refresh_when_description_is_placeholder() {
        let mut info = build_video_info(Some(185), Some("PT3M5S"));
        info.description = Some(
            "Auf YouTube findest du die angesagtesten Videos und Tracks. Außerdem kannst du eigene Inhalte hochladen und mit Freunden oder gleich der ganzen Welt teilen".to_string(),
        );
        assert!(cached_video_info_needs_refresh(&info));
    }

    #[test]
    fn enrich_video_info_fills_missing_fields_from_video() {
        let video = build_video();
        let mut info = build_video_info(Some(185), Some("PT3M5S"));
        info.title.clear();

        enrich_video_info(&mut info, &video);

        assert_eq!(info.title, video.title);
        assert_eq!(info.thumbnail_url, video.thumbnail_url);
        assert_eq!(info.published_at, Some(video.published_at));
    }

    #[test]
    fn enrich_video_info_preserves_fetched_fields_when_present() {
        let video = build_video();
        let published_at = Utc::now() - chrono::Duration::days(3);
        let mut info = VideoInfo {
            video_id: "video-123".to_string(),
            watch_url: "https://www.youtube.com/watch?v=video-123".to_string(),
            title: "Fetched Title".to_string(),
            description: None,
            thumbnail_url: Some("https://example.com/fetched-thumb.jpg".to_string()),
            channel_name: None,
            channel_id: None,
            published_at: Some(published_at),
            duration_iso8601: Some("PT3M5S".to_string()),
            duration_seconds: Some(185),
            view_count: None,
        };

        enrich_video_info(&mut info, &video);

        assert_eq!(info.title, "Fetched Title");
        assert_eq!(
            info.thumbnail_url,
            Some("https://example.com/fetched-thumb.jpg".to_string())
        );
        assert_eq!(info.published_at, Some(published_at));
    }

    #[test]
    fn video_list_params_resolve_limits_and_filters() {
        let params = VideoListParams {
            limit: Some(500),
            offset: Some(7),
            include_shorts: Some(false),
            video_type: None,
            acknowledged: Some(true),
            queue_only: Some(true),
            queue_tab: None,
        };

        assert_eq!(params.limit_or_default(), 100);
        assert_eq!(params.offset_or_default(), 7);
        assert_eq!(params.is_short_filter(), Some(false));
        assert_eq!(params.acknowledged_filter(), Some(true));
        assert_eq!(params.queue_filter(), Some(db::QueueFilter::AnyIncomplete));
    }

    #[test]
    fn video_list_params_prefer_explicit_queue_tab() {
        let params = VideoListParams {
            limit: None,
            offset: None,
            include_shorts: Some(true),
            video_type: Some(VideoTypeFilter::Short),
            acknowledged: None,
            queue_only: Some(true),
            queue_tab: Some(QueueTab::Summaries),
        };

        assert_eq!(params.limit_or_default(), 20);
        assert_eq!(params.offset_or_default(), 0);
        assert_eq!(params.is_short_filter(), Some(true));
        assert_eq!(params.queue_filter(), Some(db::QueueFilter::SummariesOnly));
    }
}
