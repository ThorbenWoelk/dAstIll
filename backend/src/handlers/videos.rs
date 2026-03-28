use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use std::collections::HashSet;

use crate::audit;
use crate::db;
use crate::models::{
    AddVideoRequest, AddVideoResponse, ChannelVideoPagePayload, ContentStatus, OTHERS_CHANNEL_ID,
    UserVideoMembership, UserVideoState, Video, VideoInfo,
};
use crate::security::{AccessContext, AuthState};
use crate::state::AppState;

fn resolve_manual_video_target_channel_id(
    video_channel_id: &str,
    subscribed_channel_ids: &HashSet<String>,
) -> String {
    if subscribed_channel_ids.contains(video_channel_id) {
        video_channel_id.to_string()
    } else {
        OTHERS_CHANNEL_ID.to_string()
    }
}

fn is_valid_youtube_video_id(candidate: &str) -> bool {
    candidate.len() == 11
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn parse_manual_video_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if is_valid_youtube_video_id(trimmed) {
        return Some(trimmed.to_string());
    }

    let url = reqwest::Url::parse(trimmed).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();

    if host == "youtu.be" {
        return url
            .path_segments()?
            .next()
            .filter(|segment| is_valid_youtube_video_id(segment))
            .map(str::to_string);
    }

    let is_youtube_host = matches!(
        host.as_str(),
        "youtube.com" | "www.youtube.com" | "m.youtube.com"
    );
    if !is_youtube_host {
        return None;
    }

    let path = url.path().trim_matches('/');
    if path == "watch" {
        return url
            .query_pairs()
            .find_map(|(key, value)| (key == "v").then(|| value.into_owned()))
            .filter(|value| is_valid_youtube_video_id(value));
    }

    let mut segments = path.split('/');
    match segments.next() {
        Some("shorts" | "embed" | "live") => segments
            .next()
            .filter(|segment| is_valid_youtube_video_id(segment))
            .map(str::to_string),
        _ => None,
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
    if info.duration_seconds.is_none()
        && info
            .duration_iso8601
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return true;
    }
    info.description.as_deref().is_some_and(|d| {
        crate::services::youtube::placeholder::is_site_wide_placeholder_description(d)
    })
}
use super::query::VideoListParams;
use super::{
    evict_video_scope_cache, map_db_err, require_channel_for_access, require_present,
    require_video_for_access,
};

#[derive(Debug, serde::Deserialize)]
pub struct VideoInfoBackfillParams {
    pub limit: Option<usize>,
    pub force: Option<bool>,
    /// Re-fetch stored video info rows whose description is a known YouTube site-wide blurb.
    pub heal_placeholders: Option<bool>,
}

pub async fn list_channel_videos(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(channel_id): Path<String>,
    Query(params): Query<VideoListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    require_channel_for_access(&state, &access_context, &channel_id).await?;

    tracing::debug!(video_type = ?params.video_type, "list_channel_videos filter");
    let page = db::list_user_scoped_videos_by_channel(
        &state.db,
        access_context.user_id.as_deref(),
        &channel_id,
        &access_context.allowed_channel_ids,
        &access_context.allowed_other_video_ids,
        params.limit_or_default(),
        params.offset_or_default(),
        params.is_short_filter(),
        params.acknowledged_filter(),
        params.queue_filter(),
    )
    .await
    .map_err(map_db_err)?;
    let page = page.ok_or((StatusCode::NOT_FOUND, "Channel not found".to_string()))?;

    Ok(Json(ChannelVideoPagePayload {
        videos: page.videos,
        has_more: page.has_more,
        next_offset: page.next_offset,
    }))
}

pub async fn add_manual_video(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<AddVideoRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    let input = payload.input.trim();
    let video_id = parse_manual_video_id(input).ok_or((
        StatusCode::BAD_REQUEST,
        "Enter a YouTube video URL, shorts URL, or 11-character video ID.".to_string(),
    ))?;

    let subscribed_channel_ids = access_context
        .allowed_channel_ids
        .iter()
        .cloned()
        .collect::<HashSet<_>>();

    if let Some(existing_video) = db::get_video(&state.db, &video_id, false)
        .await
        .map_err(map_db_err)?
    {
        db::put_user_video_membership(
            &state.db,
            user_id,
            &UserVideoMembership {
                video_id: existing_video.id.clone(),
                added_at: Utc::now(),
            },
        )
        .await
        .map_err(map_db_err)?;
        let target_channel_id = resolve_manual_video_target_channel_id(
            &existing_video.channel_id,
            &subscribed_channel_ids,
        );
        audit::log_manual_video_add(
            user_id,
            &video_id,
            &existing_video.channel_id,
            &target_channel_id,
            true,
        );
        return Ok((
            StatusCode::OK,
            Json(AddVideoResponse {
                video: existing_video,
                target_channel_id,
                already_exists: true,
            }),
        ));
    }

    let info = state
        .youtube
        .fetch_video_info(&video_id)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;
    let channel_id = info.channel_id.clone().ok_or((
        StatusCode::BAD_REQUEST,
        "Could not determine the video's channel.".to_string(),
    ))?;
    let is_short = state.youtube.fetch_is_short_flag(&video_id).await;
    let video = Video {
        id: video_id.clone(),
        channel_id: channel_id.clone(),
        title: info.title.clone(),
        thumbnail_url: info.thumbnail_url.clone(),
        published_at: info.published_at.unwrap_or_else(Utc::now),
        is_short,
        transcript_status: ContentStatus::Pending,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    };

    db::insert_video(&state.db, &video)
        .await
        .map_err(map_db_err)?;
    db::upsert_video_info(&state.db, &info)
        .await
        .map_err(map_db_err)?;
    db::put_user_video_membership(
        &state.db,
        user_id,
        &UserVideoMembership {
            video_id: video_id.clone(),
            added_at: Utc::now(),
        },
    )
    .await
    .map_err(map_db_err)?;

    let target_channel_id =
        resolve_manual_video_target_channel_id(&channel_id, &subscribed_channel_ids);
    audit::log_manual_video_add(user_id, &video_id, &channel_id, &target_channel_id, false);
    evict_video_scope_cache(&state, &channel_id).await?;
    state.read_cache.evict_channel_list().await;

    Ok((
        StatusCode::CREATED,
        Json(AddVideoResponse {
            video,
            target_channel_id,
            already_exists: false,
        }),
    ))
}

pub async fn get_video(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(
        require_video_for_access(&state, &access_context, &video_id).await?,
    ))
}

pub async fn get_video_info(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let video = require_video_for_access(&state, &access_context, &video_id).await?;

    let cached = {
        db::get_video_info(&state.db, &video_id)
            .await
            .map_err(map_db_err)?
    };

    let mut cached = require_present(cached, "Video info not found")?;
    enrich_video_info(&mut cached, &video);
    Ok(Json(cached))
}

pub async fn ensure_video_info(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let video = require_video_for_access(&state, &access_context, &video_id).await?;

    let cached = {
        db::get_video_info(&state.db, &video_id)
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
            db::upsert_video_info(&state.db, &info)
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
        if heal {
            db::list_video_ids_with_placeholder_descriptions(&state.db, limit)
                .await
                .map_err(map_db_err)?
        } else if force {
            db::list_video_ids_for_info_refresh(&state.db, limit)
                .await
                .map_err(map_db_err)?
        } else {
            db::list_video_ids_missing_info(&state.db, limit)
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
            db::get_video(&state.db, video_id, false)
                .await
                .map_err(map_db_err)?
        };

        if let Some(video) = video {
            enrich_video_info(&mut info, &video);
        }

        match db::upsert_video_info(&state.db, &info).await {
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
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
    Json(payload): Json<crate::models::UpdateAcknowledgedRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }
    let mut video = require_video_for_access(&state, &access_context, &video_id).await?;
    let old_acknowledged = video.acknowledged;
    db::put_user_video_state(
        &state.db,
        user_id,
        &UserVideoState {
            video_id: video_id.clone(),
            acknowledged: payload.acknowledged,
            updated_at: Utc::now(),
        },
    )
    .await
    .map_err(map_db_err)?;
    video.acknowledged = payload.acknowledged;
    audit::log_video_acknowledgment(
        user_id,
        &video_id,
        &video.channel_id,
        old_acknowledged,
        payload.acknowledged,
    );
    evict_video_scope_cache(&state, &video.channel_id).await?;
    Ok(Json(video))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::Utc;

    use super::{
        VideoListParams, cached_video_info_needs_refresh, enrich_video_info,
        resolve_manual_video_target_channel_id,
    };
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

    #[test]
    fn manual_video_targets_subscribed_channel_when_available() {
        let subscribed = HashSet::from(["UC_SUBSCRIBED".to_string()]);

        assert_eq!(
            resolve_manual_video_target_channel_id("UC_SUBSCRIBED", &subscribed),
            "UC_SUBSCRIBED"
        );
    }

    #[test]
    fn manual_video_targets_others_when_channel_is_not_subscribed() {
        let subscribed = HashSet::from(["UC_SUBSCRIBED".to_string()]);

        assert_eq!(
            resolve_manual_video_target_channel_id("UC_OTHER", &subscribed),
            "__others__"
        );
    }
}
