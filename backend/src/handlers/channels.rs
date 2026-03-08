use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashSet;

use crate::db;
use crate::handlers::videos::VideoTypeFilter;
use crate::models::{AddChannelRequest, Channel, UpdateChannelRequest};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct BackfillParams {
    pub limit: Option<usize>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelSnapshotParams {
    pub selected_channel_id: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_shorts: Option<bool>,
    pub video_type: Option<VideoTypeFilter>,
    pub acknowledged: Option<bool>,
    pub queue_only: Option<bool>,
}

fn resolve_is_short(params: &ChannelSnapshotParams) -> Option<bool> {
    match params.video_type {
        Some(video_type) => video_type.as_is_short(),
        None => {
            let include_shorts = params.include_shorts.unwrap_or(true);
            if include_shorts { None } else { Some(false) }
        }
    }
}

fn build_sync_depth_payload(
    channel: &Channel,
    derived_earliest_ready_date: Option<chrono::DateTime<chrono::Utc>>,
) -> crate::models::SyncDepthPayload {
    crate::models::SyncDepthPayload {
        earliest_sync_date: channel.earliest_sync_date.map(|dt| dt.to_rfc3339()),
        earliest_sync_date_user_set: channel.earliest_sync_date_user_set,
        derived_earliest_ready_date: derived_earliest_ready_date.map(|dt| dt.to_rfc3339()),
    }
}

fn build_snapshot_payload(
    snapshot: db::ChannelSnapshotData,
) -> crate::models::ChannelSnapshotPayload {
    crate::models::ChannelSnapshotPayload {
        channel_id: snapshot.channel.id.clone(),
        sync_depth: build_sync_depth_payload(
            &snapshot.channel,
            snapshot.derived_earliest_ready_date,
        ),
        videos: snapshot.videos,
    }
}

pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let channels = db::list_channels(&conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(channels))
}

pub async fn workspace_bootstrap(
    State(state): State<AppState>,
    Query(params): Query<ChannelSnapshotParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let is_short = resolve_is_short(&params);
    let queue_only = params.queue_only.unwrap_or(false);

    let ai_available = state.summarizer.is_available().await;
    let ai_status = state
        .summarizer
        .indicator_status(state.cloud_cooldown.is_active(), ai_available);
    let conn = state.db.lock().await;
    let bootstrap = db::load_workspace_bootstrap_data(
        &conn,
        params.selected_channel_id.as_deref(),
        limit,
        offset,
        is_short,
        params.acknowledged,
        queue_only,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(crate::models::WorkspaceBootstrapPayload {
        ai_available,
        ai_status,
        channels: bootstrap.channels,
        selected_channel_id: bootstrap.selected_channel_id,
        snapshot: bootstrap.snapshot.map(build_snapshot_payload),
    }))
}

pub async fn add_channel(
    State(state): State<AppState>,
    Json(payload): Json<AddChannelRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let input = payload.input.trim().to_string();
    let youtube = state.youtube.clone();

    let (channel_id, name, resolved_thumbnail) = youtube
        .resolve_channel(&input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    tracing::info!(channel_id = %channel_id, input = %input, "resolved channel input");

    let thumbnail = match youtube.fetch_channel_thumbnail(&channel_id).await {
        Ok(Some(url)) => Some(url),
        Ok(None) | Err(_) => resolved_thumbnail,
    };

    let handle = if input.starts_with('@') {
        Some(input.clone())
    } else if !input.starts_with("http") && !input.starts_with("UC") {
        Some(format!("@{input}"))
    } else {
        None
    };

    let earliest_sync_date = match youtube.fetch_videos(&channel_id).await {
        Ok(videos) if !videos.is_empty() => Some(videos[0].published_at),
        _ => Some(Utc::now()),
    };

    let channel = Channel {
        id: channel_id.clone(),
        handle,
        name,
        thumbnail_url: thumbnail,
        added_at: Utc::now(),
        earliest_sync_date,
        earliest_sync_date_user_set: false,
    };

    {
        let conn = state.db.lock().await;
        db::insert_channel(&conn, &channel)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    tracing::info!(channel_id = %channel.id, channel_name = %channel.name, "channel subscribed");

    let db_pool = state.db.clone();
    let channel_id_clone = channel_id.clone();
    tokio::spawn(async move {
        match youtube.fetch_videos(&channel_id_clone).await {
            Ok(videos) => {
                let conn = db_pool.lock().await;
                let mut inserted_count = 0;
                for video in videos {
                    if matches!(
                        crate::db::insert_video(&conn, &video).await,
                        Ok(crate::db::VideoInsertOutcome::Inserted)
                    ) {
                        inserted_count += 1;
                    }
                }
                tracing::info!(
                    channel_id = %channel_id_clone,
                    inserted_count,
                    "subscribed channel initial sync inserted new videos"
                );
            }
            Err(err) => {
                tracing::warn!(
                    channel_id = %channel_id_clone,
                    error = %err,
                    "failed to fetch videos after subscribing channel"
                );
            }
        }
    });

    Ok((StatusCode::CREATED, Json(channel)))
}

pub async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let channel = db::get_channel(&conn, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match channel {
        Some(c) => Ok(Json(c)),
        None => Err((StatusCode::NOT_FOUND, "Channel not found".to_string())),
    }
}

pub async fn get_channel_sync_depth(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let channel = db::get_channel(&conn, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let channel = match channel {
        Some(c) => c,
        None => return Err((StatusCode::NOT_FOUND, "Channel not found".to_string())),
    };

    let derived = db::get_oldest_ready_video_published_at(&conn, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(build_sync_depth_payload(&channel, derived)))
}

pub async fn get_channel_snapshot(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<ChannelSnapshotParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let is_short = resolve_is_short(&params);
    let queue_only = params.queue_only.unwrap_or(false);

    let conn = state.db.lock().await;
    let snapshot = db::load_channel_snapshot_data(
        &conn,
        &id,
        limit,
        offset,
        is_short,
        params.acknowledged,
        queue_only,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match snapshot {
        Some(snapshot) => Ok(Json(build_snapshot_payload(snapshot))),
        None => Err((StatusCode::NOT_FOUND, "Channel not found".to_string())),
    }
}

pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.lock().await;
    let deleted = db::delete_channel(&conn, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Channel not found".to_string()))
    }
}

pub async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut channel = {
        let conn = state.db.lock().await;
        db::get_channel(&conn, &id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Channel not found".to_string()))?
    };

    if let Some(v) = payload.earliest_sync_date {
        channel.earliest_sync_date = Some(v);
    }
    if let Some(v) = payload.earliest_sync_date_user_set {
        channel.earliest_sync_date_user_set = v;
    }

    {
        let conn = state.db.lock().await;
        db::insert_channel(&conn, &channel)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(channel))
}

const REFRESH_BACKFILL_BATCH: usize = 50;
const REFRESH_BACKFILL_MAX_ROUNDS: usize = 20;

pub async fn refresh_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "refresh requested - queueing latest videos");

    let earliest_sync_date = {
        let conn = state.db.lock().await;
        let channel = db::get_channel(&conn, &id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        match channel {
            Some(c) => c.earliest_sync_date,
            None => return Err((StatusCode::NOT_FOUND, "Channel not found".to_string())),
        }
    };

    let videos = state
        .youtube
        .fetch_videos(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut count = 0;
    {
        let conn = state.db.lock().await;
        for video in videos {
            if matches!(
                db::insert_video(&conn, &video).await,
                Ok(db::VideoInsertOutcome::Inserted)
            ) {
                count += 1;
            }
        }
    }

    if let Some(until) = earliest_sync_date {
        for round in 0..REFRESH_BACKFILL_MAX_ROUNDS {
            let known_video_ids = {
                let conn = state.db.lock().await;
                db::list_video_ids_by_channel(&conn, &id)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                    .into_iter()
                    .collect::<HashSet<_>>()
            };

            let (backfill_videos, exhausted) = state
                .youtube
                .fetch_videos_backfill_missing(
                    &id,
                    &known_video_ids,
                    REFRESH_BACKFILL_BATCH,
                    Some(until),
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let added = {
                let conn = state.db.lock().await;
                let mut n = 0;
                for video in backfill_videos {
                    if matches!(
                        db::insert_video(&conn, &video).await,
                        Ok(db::VideoInsertOutcome::Inserted)
                    ) {
                        n += 1;
                    }
                }
                n
            };

            count += added;
            if added > 0 {
                tracing::info!(
                    channel_id = %id,
                    round = round + 1,
                    added,
                    "refresh backfill round"
                );
            }

            if added == 0 || exhausted {
                break;
            }
        }
    }

    tracing::info!(
        channel_id = %id,
        inserted_count = count,
        "channel refresh inserted new videos"
    );

    Ok(Json(serde_json::json!({ "videos_added": count })))
}

pub async fn backfill_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<BackfillParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "backfill requested");

    let batch_limit = params.limit.unwrap_or(15).clamp(1, 100);

    let known_video_ids = {
        let conn = state.db.lock().await;
        let channel = db::get_channel(&conn, &id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if channel.is_none() {
            return Err((StatusCode::NOT_FOUND, "Channel not found".to_string()));
        }

        db::list_video_ids_by_channel(&conn, &id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .into_iter()
            .collect::<HashSet<_>>()
    };
    let known_count = known_video_ids.len();

    let (videos, exhausted) = state
        .youtube
        .fetch_videos_backfill_missing(&id, &known_video_ids, batch_limit, params.until)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let fetched_count = videos.len();
    let conn = state.db.lock().await;

    let mut added_count = 0;
    for video in videos {
        if matches!(
            db::insert_video(&conn, &video).await,
            Ok(db::VideoInsertOutcome::Inserted)
        ) {
            added_count += 1;
        }
    }

    tracing::info!(
        channel_id = %id,
        known_count,
        fetched_count,
        added_count,
        exhausted,
        "channel history backfill complete"
    );

    Ok(Json(serde_json::json!({
        "videos_added": added_count,
        "fetched_count": fetched_count,
        "exhausted": exhausted
    })))
}
