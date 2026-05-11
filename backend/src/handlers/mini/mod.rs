use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::{sync::Semaphore, task::JoinSet};
use utoipa::{IntoParams, ToSchema};

use crate::{
    audit, db,
    models::{Channel, Summary, UserVideoState, VideoInfo},
    security::{AccessContext, AuthState},
    state::AppState,
};

use super::map_db_err;

#[derive(Debug, Deserialize, IntoParams)]
pub struct MiniReaderParams {
    pub channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MiniSummaryItem {
    pub video_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub title: String,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    pub watch_url: String,
    pub summary_content: String,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MiniReaderPayload {
    pub channels: Vec<Channel>,
    #[serde(default)]
    pub selected_channel_id: Option<String>,
    pub summaries: Vec<MiniSummaryItem>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMiniReadStatusRequest {
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateMiniReadStatusResponse {
    pub video_id: String,
    pub read: bool,
    pub updated_at: DateTime<Utc>,
}

fn require_authenticated_user(
    access_context: &AccessContext,
) -> Result<&str, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }
    Ok(user_id)
}

fn build_mini_summary_item(
    video: &crate::models::Video,
    summary: &Summary,
    video_info: Option<&VideoInfo>,
    channel_name_by_id: &HashMap<String, String>,
) -> MiniSummaryItem {
    let channel_id = video.channel_id.clone();
    MiniSummaryItem {
        video_id: video.id.clone(),
        channel_id: channel_id.clone(),
        channel_name: video_info
            .and_then(|item| item.channel_name.clone())
            .or_else(|| channel_name_by_id.get(&channel_id).cloned())
            .unwrap_or(channel_id.clone()),
        title: if video.title.trim().is_empty() {
            video.id.clone()
        } else {
            video.title.clone()
        },
        thumbnail_url: video_info
            .and_then(|item| item.thumbnail_url.clone())
            .or_else(|| video.thumbnail_url.clone()),
        published_at: video_info
            .and_then(|item| item.published_at)
            .or(Some(video.published_at)),
        watch_url: video_info
            .map(|item| item.watch_url.clone())
            .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", video.id)),
        summary_content: summary.content.clone(),
        read: video.acknowledged,
    }
}

async fn load_summary_items_for_videos(
    store: &db::Store,
    videos: Vec<crate::models::Video>,
    channel_name_by_id: Arc<HashMap<String, String>>,
) -> Result<Vec<MiniSummaryItem>, db::StoreError> {
    if videos.is_empty() {
        return Ok(Vec::new());
    }

    let semaphore = Arc::new(Semaphore::new(db::MAX_CONCURRENT_S3_OPS));
    let mut join_set: JoinSet<Result<Option<MiniSummaryItem>, db::StoreError>> = JoinSet::new();

    for video in videos {
        let store = store.clone();
        let semaphore = Arc::clone(&semaphore);
        let channel_name_by_id = Arc::clone(&channel_name_by_id);
        join_set.spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");
            let Some(summary) = db::get_summary(&store, &video.id).await? else {
                return Ok(None);
            };
            if summary.content.trim().is_empty() {
                return Ok(None);
            }
            let video_info = db::get_video_info(&store, &video.id).await?;
            Ok(Some(build_mini_summary_item(
                &video,
                &summary,
                video_info.as_ref(),
                channel_name_by_id.as_ref(),
            )))
        });
    }

    let mut items = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(Some(item))) => items.push(item),
            Ok(Ok(None)) => {}
            Ok(Err(err)) => return Err(err),
            Err(err) => {
                return Err(db::StoreError::Other(format!(
                    "mini summary load task failed: {err}"
                )));
            }
        }
    }

    Ok(items)
}

async fn load_reader_payload(
    store: &db::Store,
    user_id: &str,
    selected_channel_id: Option<&str>,
) -> Result<MiniReaderPayload, db::StoreError> {
    let channels = db::list_user_channels(store, user_id).await?;
    let selected_channel_id = selected_channel_id
        .and_then(|candidate| {
            channels
                .iter()
                .find(|channel| channel.id == candidate)
                .map(|channel| channel.id.clone())
        })
        .or_else(|| channels.first().map(|channel| channel.id.clone()));

    let Some(selected_channel_id_value) = selected_channel_id.clone() else {
        return Ok(MiniReaderPayload {
            channels,
            selected_channel_id: None,
            summaries: Vec::new(),
        });
    };
    let allowed_channel_ids = channels
        .iter()
        .map(|channel| channel.id.clone())
        .collect::<Vec<_>>();
    let channel_name_by_id = Arc::new(
        channels
            .iter()
            .map(|channel| (channel.id.clone(), channel.name.clone()))
            .collect::<HashMap<_, _>>(),
    );
    let page = db::list_user_scoped_videos_by_channel(
        store,
        Some(user_id),
        &selected_channel_id_value,
        &allowed_channel_ids,
        &[],
        500,
        0,
        Some(false),
        None,
        None,
    )
    .await?;
    let Some(page) = page else {
        return Ok(MiniReaderPayload {
            channels,
            selected_channel_id: Some(selected_channel_id_value),
            summaries: Vec::new(),
        });
    };
    let mut summary_items =
        load_summary_items_for_videos(store, page.videos, channel_name_by_id).await?;

    summary_items.sort_by(|left, right| {
        right
            .published_at
            .cmp(&left.published_at)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.video_id.cmp(&right.video_id))
    });

    Ok(MiniReaderPayload {
        channels,
        selected_channel_id: Some(selected_channel_id_value),
        summaries: summary_items,
    })
}

#[utoipa::path(
    get,
    path = "/api/mini",
    params(MiniReaderParams),
    responses(
        (status = 200, description = "Minimal reader payload", body = MiniReaderPayload),
        (status = 403, description = "Sign-in required", body = String)
    ),
    tag = "Mini"
)]
pub async fn get_mini_reader(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Query(params): Query<MiniReaderParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_authenticated_user(&access_context)?;
    let payload = load_reader_payload(&state.db, user_id, params.channel_id.as_deref())
        .await
        .map_err(map_db_err)?;
    Ok(Json(payload))
}

#[utoipa::path(
    put,
    path = "/api/mini/videos/{id}/read",
    params(
        ("id" = String, Path, description = "Video id")
    ),
    request_body = UpdateMiniReadStatusRequest,
    responses(
        (status = 200, description = "Updated mini reader read status", body = UpdateMiniReadStatusResponse),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Summary not found", body = String)
    ),
    tag = "Mini"
)]
pub async fn update_mini_read_status(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(video_id): Path<String>,
    Json(payload): Json<UpdateMiniReadStatusRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_authenticated_user(&access_context)?;

    let summary = db::get_summary(&state.db, &video_id)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Summary not found".to_string()))?;
    if summary.content.trim().is_empty() {
        return Err((StatusCode::NOT_FOUND, "Summary not found".to_string()));
    }

    let video = db::get_video(&state.db, &video_id, false)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Summary not found".to_string()))?;
    let channel_id = video.channel_id.clone();

    let has_subscription = db::get_user_channel(&state.db, user_id, &channel_id)
        .await
        .map_err(map_db_err)?
        .is_some();
    if !has_subscription {
        return Err((StatusCode::NOT_FOUND, "Summary not found".to_string()));
    }

    let old_acknowledged = db::get_user_video_state(&state.db, user_id, &video_id)
        .await
        .map_err(map_db_err)?
        .map(|state| state.acknowledged)
        .unwrap_or(false);
    let updated_at = Utc::now();
    db::put_user_video_state(
        &state.db,
        user_id,
        &UserVideoState {
            video_id: video_id.clone(),
            acknowledged: payload.read,
            updated_at,
        },
    )
    .await
    .map_err(map_db_err)?;

    audit::log_video_acknowledgment(
        user_id,
        &video_id,
        &channel_id,
        old_acknowledged,
        payload.read,
    );

    Ok(Json(UpdateMiniReadStatusResponse {
        video_id,
        read: payload.read,
        updated_at,
    }))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
