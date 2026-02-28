use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;

use crate::db;
use crate::models::{AddChannelRequest, Channel};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct BackfillParams {
    pub limit: Option<usize>,
}

pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state
        .db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let channels =
        db::list_channels(&conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(channels))
}

pub async fn add_channel(
    State(state): State<AppState>,
    Json(payload): Json<AddChannelRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let input = payload.input.trim().to_string();
    let youtube = state.youtube.clone();

    // Resolve the input to a channel ID and name
    let (channel_id, name, resolved_thumbnail) = youtube
        .resolve_channel(&input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    tracing::info!(channel_id = %channel_id, input = %input, "resolved channel input");

    // Always attempt to fetch the canonical channel avatar on subscribe.
    let thumbnail = match youtube.fetch_channel_thumbnail(&channel_id).await {
        Ok(Some(url)) => Some(url),
        Ok(None) | Err(_) => resolved_thumbnail,
    };

    // Extract handle from input if it looks like one
    let handle = if input.starts_with('@') {
        Some(input.clone())
    } else if !input.starts_with("http") && !input.starts_with("UC") {
        Some(format!("@{input}"))
    } else {
        None
    };

    let channel = Channel {
        id: channel_id.clone(),
        handle,
        name,
        thumbnail_url: thumbnail,
        added_at: Utc::now(),
    };

    // Save to database
    {
        let conn = state
            .db
            .lock()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        db::insert_channel(&conn, &channel)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    tracing::info!(channel_id = %channel.id, channel_name = %channel.name, "channel subscribed");

    // Fetch and store recent videos
    match youtube.fetch_videos(&channel_id).await {
        Ok(videos) => {
            let conn = state
                .db
                .lock()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let mut queued_count = 0;
            for video in videos {
                if db::insert_video(&conn, &video).is_ok() {
                    queued_count += 1;
                }
            }
            tracing::info!(channel_id = %channel_id, queued_count, "queued channel videos");
        }
        Err(err) => {
            tracing::warn!(
                channel_id = %channel_id,
                error = %err,
                "failed to fetch videos after subscribing channel"
            );
        }
    }

    Ok((StatusCode::CREATED, Json(channel)))
}

pub async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state
        .db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let channel = db::get_channel(&conn, &id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match channel {
        Some(c) => Ok(Json(c)),
        None => Err((StatusCode::NOT_FOUND, "Channel not found".to_string())),
    }
}

pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state
        .db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let deleted = db::delete_channel(&conn, &id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Channel not found".to_string()))
    }
}

pub async fn refresh_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "refresh requested - queueing latest videos");
    // Verify channel exists
    {
        let conn = state
            .db
            .lock()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let channel = db::get_channel(&conn, &id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if channel.is_none() {
            return Err((StatusCode::NOT_FOUND, "Channel not found".to_string()));
        }
    }

    let videos = state
        .youtube
        .fetch_videos(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let conn = state
        .db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut count = 0;
    for video in videos {
        if db::insert_video(&conn, &video).is_ok() {
            count += 1;
        }
    }
    tracing::info!(channel_id = %id, queued_count = count, "channel refresh queued videos");

    Ok(Json(serde_json::json!({ "videos_added": count })))
}

pub async fn backfill_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<BackfillParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "backfill requested");

    let batch_limit = params.limit.unwrap_or(15).clamp(1, 50);

    let known_count = {
        let conn = state
            .db
            .lock()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let channel = db::get_channel(&conn, &id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if channel.is_none() {
            return Err((StatusCode::NOT_FOUND, "Channel not found".to_string()));
        }

        db::count_videos_by_channel(&conn, &id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    let videos = state
        .youtube
        .fetch_videos_backfill(&id, known_count, batch_limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let fetched_count = videos.len();
    let conn = state
        .db
        .lock()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut added_count = 0;
    for video in videos {
        if db::insert_video(&conn, &video).is_ok() {
            added_count += 1;
        }
    }

    tracing::info!(
        channel_id = %id,
        known_count,
        fetched_count,
        added_count,
        "channel history backfill complete"
    );

    Ok(Json(serde_json::json!({
        "videos_added": added_count,
        "fetched_count": fetched_count
    })))
}
