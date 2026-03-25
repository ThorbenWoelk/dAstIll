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
use crate::handlers::query::{VideoListParams, WorkspaceBootstrapParams};
use crate::models::{AddChannelRequest, Channel, UpdateChannelRequest};
use crate::read_cache::{ChannelSnapshotCacheKey, VideoListCacheKey, WorkspaceBootstrapCacheKey};
use crate::state::AppState;

use super::{map_db_err, require_channel};

#[derive(Deserialize)]
pub struct BackfillParams {
    pub limit: Option<usize>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
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
        channel_video_count: snapshot.channel_video_count,
        videos: snapshot.videos,
    }
}

pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(channels) = state.read_cache.get_channels().await {
        tracing::debug!("channels cache hit");
        return Ok(Json(channels));
    }

    let channels = db::list_channels_with_virtual_others(&state.db)
        .await
        .map_err(map_db_err)?;
    state.read_cache.set_channels(channels.clone()).await;
    Ok(Json(channels))
}

pub async fn workspace_bootstrap(
    State(state): State<AppState>,
    Query(params): Query<WorkspaceBootstrapParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let video_params = params.video_params();
    let cache_key = WorkspaceBootstrapCacheKey {
        selected_channel_id: params.selected_channel_id.clone(),
        video_list: VideoListCacheKey::new(
            video_params.limit_or_default(),
            video_params.offset_or_default(),
            video_params.is_short_filter(),
            video_params.acknowledged_filter(),
            video_params.queue_filter(),
        ),
    };
    if let Some(payload) = state.read_cache.get_workspace_bootstrap(&cache_key).await {
        tracing::debug!("workspace bootstrap cache hit");
        return Ok(Json(payload));
    }

    let ai_available = state.summarizer.is_available().await;
    let ai_status = state
        .summarizer
        .indicator_status(state.cloud_cooldown.is_active(), ai_available);
    let bootstrap = db::load_workspace_bootstrap_data(
        &state.db,
        params.selected_channel_id.as_deref(),
        video_params.limit_or_default(),
        video_params.offset_or_default(),
        video_params.is_short_filter(),
        video_params.acknowledged_filter(),
        video_params.queue_filter(),
    )
    .await
    .map_err(map_db_err)?;
    let search_status = super::search::load_search_status_payload(&state);

    let payload = crate::models::WorkspaceBootstrapPayload {
        ai_available,
        ai_status,
        channels: bootstrap.channels,
        selected_channel_id: bootstrap.selected_channel_id,
        snapshot: bootstrap.snapshot.map(build_snapshot_payload),
        search_status,
    };
    state
        .read_cache
        .set_workspace_bootstrap(cache_key, payload.clone())
        .await;

    Ok(Json(payload))
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
        db::insert_channel(&state.db, &channel)
            .await
            .map_err(map_db_err)?;
    }
    state.read_cache.evict_channel_list().await;
    state
        .read_cache
        .evict_channel(crate::models::OTHERS_CHANNEL_ID)
        .await;
    tracing::info!(channel_id = %channel.id, channel_name = %channel.name, "channel subscribed");

    let db_pool = state.db.clone();
    let read_cache = state.read_cache.clone();
    let channel_id_clone = channel_id.clone();
    tokio::spawn(async move {
        match youtube.fetch_videos(&channel_id_clone).await {
            Ok(videos) => {
                let inserted_count = crate::db::bulk_insert_videos(&db_pool, videos)
                    .await
                    .unwrap_or(0);
                read_cache.evict_channel(&channel_id_clone).await;
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
    Ok(Json(require_channel(&state, &id).await?))
}

pub async fn get_channel_sync_depth(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(payload) = state.read_cache.get_channel_sync_depth(&id).await {
        tracing::debug!(channel_id = %id, "channel sync depth cache hit");
        return Ok(Json(payload));
    }

    let channel = require_channel(&state, &id).await?;

    let derived = db::get_oldest_ready_video_published_at(&state.db, &channel)
        .await
        .map_err(map_db_err)?;

    let payload = build_sync_depth_payload(&channel, derived);
    state
        .read_cache
        .set_channel_sync_depth(id.clone(), payload.clone())
        .await;

    Ok(Json(payload))
}

pub async fn get_channel_snapshot(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<VideoListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cache_key = ChannelSnapshotCacheKey {
        channel_id: id.clone(),
        video_list: VideoListCacheKey::new(
            params.limit_or_default(),
            params.offset_or_default(),
            params.is_short_filter(),
            params.acknowledged_filter(),
            params.queue_filter(),
        ),
    };
    if let Some(payload) = state.read_cache.get_channel_snapshot(&cache_key).await {
        tracing::debug!(channel_id = %id, "channel snapshot cache hit");
        return Ok(Json(payload));
    }

    let snapshot = db::load_channel_snapshot_data(
        &state.db,
        &id,
        params.limit_or_default(),
        params.offset_or_default(),
        params.is_short_filter(),
        params.acknowledged_filter(),
        params.queue_filter(),
    )
    .await
    .map_err(map_db_err)?;

    match snapshot {
        Some(snapshot) => {
            let payload = build_snapshot_payload(snapshot);
            state
                .read_cache
                .set_channel_snapshot(cache_key, payload.clone())
                .await;
            Ok(Json(payload))
        }
        None => Err((StatusCode::NOT_FOUND, "Channel not found".to_string())),
    }
}

pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let deleted = db::delete_channel(&state.db, &id)
        .await
        .map_err(map_db_err)?;

    if deleted {
        state.read_cache.evict_channel(&id).await;
        state.read_cache.evict_channel_list().await;
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
    let mut channel = require_channel(&state, &id).await?;

    if let Some(v) = payload.earliest_sync_date {
        channel.earliest_sync_date = Some(v);
    }
    if let Some(v) = payload.earliest_sync_date_user_set {
        channel.earliest_sync_date_user_set = v;
    }

    {
        db::insert_channel(&state.db, &channel)
            .await
            .map_err(map_db_err)?;
    }
    state.read_cache.evict_channel(&id).await;
    state.read_cache.evict_channel_list().await;

    Ok(Json(channel))
}

const REFRESH_BACKFILL_BATCH: usize = 50;
const REFRESH_BACKFILL_MAX_ROUNDS: usize = 20;

pub async fn refresh_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "refresh requested - queueing latest videos");

    let earliest_sync_date = require_channel(&state, &id).await?.earliest_sync_date;

    let videos = state
        .youtube
        .fetch_videos(&id)
        .await
        .map_err(map_db_err)?;

    let mut count = {
        db::bulk_insert_videos(&state.db, videos)
            .await
            .map_err(map_db_err)?
    };

    if let Some(until) = earliest_sync_date {
        for round in 0..REFRESH_BACKFILL_MAX_ROUNDS {
            let known_video_ids = {
                db::list_video_ids_by_channel(&state.db, &id)
                    .await
                    .map_err(map_db_err)?
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
                .map_err(map_db_err)?;

            let added = {
                db::bulk_insert_videos(&state.db, backfill_videos)
                    .await
                    .map_err(map_db_err)?
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
    state.read_cache.evict_channel(&id).await;

    Ok(Json(serde_json::json!({ "videos_added": count })))
}

pub async fn backfill_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<BackfillParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "backfill requested");

    let batch_limit = params.limit.unwrap_or(15).clamp(1, 100);

    require_channel(&state, &id).await?;
    let known_video_ids = {
        db::list_video_ids_by_channel(&state.db, &id)
            .await
            .map_err(map_db_err)?
            .into_iter()
            .collect::<HashSet<_>>()
    };
    let known_count = known_video_ids.len();

    let (videos, exhausted) = state
        .youtube
        .fetch_videos_backfill_missing(&id, &known_video_ids, batch_limit, params.until)
        .await
        .map_err(map_db_err)?;

    let fetched_count = videos.len();
    let added_count = db::bulk_insert_videos(&state.db, videos)
        .await
        .map_err(map_db_err)?;

    tracing::info!(
        channel_id = %id,
        known_count,
        fetched_count,
        added_count,
        exhausted,
        "channel history backfill complete"
    );
    state.read_cache.evict_channel(&id).await;

    Ok(Json(serde_json::json!({
        "videos_added": added_count,
        "fetched_count": fetched_count,
        "exhausted": exhausted
    })))
}

// Tests require S3 backend; run with: cargo test -- --ignored
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::to_bytes,
        extract::{Query, State},
        response::IntoResponse,
    };
    use chrono::Utc;
    use reqwest::Client;
    use serde_json::Value;
    use tokio::sync::RwLock;

    use super::workspace_bootstrap;
    use crate::{
        db::{
            Store, insert_channel, insert_video, list_search_progress_materials, upsert_transcript,
        },
        handlers::query::WorkspaceBootstrapParams,
        models::{Channel, ContentStatus, Transcript, TranscriptRenderMode, Video},
        search_progress::SearchProgress,
        services::{
            ChatService, CloudCooldown, OllamaCore, SearchService, SummarizerService,
            SummaryEvaluatorService, TranscriptCooldown, TranscriptService, YouTubeQuotaCooldown,
            YouTubeService,
        },
        state::AppState,
    };

    fn test_app_state(db: crate::db::Store) -> AppState {
        let cooldown = Arc::new(CloudCooldown::cloud());
        let security =
            Arc::new(crate::config::SecurityRuntimeConfig::from_env().expect("security config"));
        AppState {
            db,
            read_cache: Arc::new(crate::read_cache::ReadCache::default()),
            security: security.clone(),
            request_rate_limiter: crate::security::rate_limiter(security.as_ref()),
            search_auto_create_vector_index: false,
            search_projection_lock: Arc::new(RwLock::new(())),
            search_progress: Arc::new(SearchProgress::new(
                None,
                crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
                false,
            )),
            youtube: Arc::new(YouTubeService::with_client(Client::new())),
            transcript: Arc::new(TranscriptService::with_path("/usr/bin/false")),
            summarizer: Arc::new(SummarizerService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            summary_evaluator: Arc::new(SummaryEvaluatorService::new(
                OllamaCore::new("://invalid-url", "qwen3.5:397b-cloud")
                    .with_cloud_cooldown(cooldown.clone()),
            )),
            search: Arc::new(SearchService::with_config(
                "://invalid-url",
                None,
                crate::services::search::SEARCH_EMBEDDING_DIMENSIONS,
                false,
            )),
            chat: Arc::new(ChatService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            analytics: None,
            active_chats: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            chat_store_lock: Arc::new(tokio::sync::Mutex::new(())),
            anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
            cloud_cooldown: cooldown,
            youtube_quota_cooldown: Arc::new(YouTubeQuotaCooldown::youtube_quota()),
            transcript_cooldown: Arc::new(TranscriptCooldown::transcript()),
        }
    }

    #[tokio::test]
    #[ignore] // requires live S3 backend
    async fn workspace_bootstrap_includes_search_status_for_initial_render() {
        let store = Store::for_test().await;
        let channel = Channel {
            id: "UC_BOOT_SEARCH".to_string(),
            handle: None,
            name: "Bootstrap Search".to_string(),
            thumbnail_url: None,
            added_at: Utc::now(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        };
        insert_channel(&store, &channel).await.unwrap();
        insert_video(
            &store,
            &Video {
                id: "vid_boot_search".to_string(),
                channel_id: channel.id.clone(),
                title: "Ready transcript".to_string(),
                thumbnail_url: None,
                published_at: Utc::now(),
                is_short: false,
                transcript_status: ContentStatus::Ready,
                summary_status: ContentStatus::Pending,
                acknowledged: false,
                retry_count: 0,
                quality_score: None,
            },
        )
        .await
        .unwrap();
        upsert_transcript(
            &store,
            &Transcript {
                video_id: "vid_boot_search".to_string(),
                raw_text: Some("bootstrap transcript content".to_string()),
                formatted_markdown: None,
                render_mode: TranscriptRenderMode::PlainText,
            },
        )
        .await
        .unwrap();

        let state = test_app_state(store.clone());
        let materials = list_search_progress_materials(&store).await.unwrap();
        state
            .search_progress
            .initialize_from_materials(&materials, false, false)
            .await;

        let response =
            workspace_bootstrap(State(state), Query(WorkspaceBootstrapParams::default()))
                .await
                .unwrap()
                .into_response();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["channels"].as_array().unwrap().len(), 1);
        assert_eq!(payload["search_status"]["total_sources"].as_u64(), Some(1));
        assert_eq!(payload["search_status"]["ready"].as_u64(), Some(0));
    }
}
