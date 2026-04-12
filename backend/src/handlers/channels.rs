use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashSet;
use utoipa::IntoParams;

use crate::audit;
use crate::db;
use crate::db::SourceProfileRecord;
use crate::handlers::query::{VideoListParams, WorkspaceBootstrapParams};
use crate::models::{
    AddChannelRequest, Channel, OpenAlexPlanRequest, OpenAlexPlanResponse, UpdateChannelRequest,
};
use crate::read_cache::{ChannelSnapshotCacheKey, VideoListCacheKey};
use crate::security::{AccessContext, AuthState};
use crate::services::{
    FeedSourceAdapter, QuerySourceAdapter, SearchSourceKind, persist_source_profile_and_channel,
    sync_source_profile,
};
use crate::state::AppState;

use super::{map_db_err, require_channel, require_channel_for_access};

#[derive(Deserialize, IntoParams)]
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

enum AddSourceIntent {
    YouTubeChannel,
    OpenAlexQuery(String),
    PodcastFeed(String),
    WebsitePage(String),
}

fn parse_add_source_intent(input: &str) -> AddSourceIntent {
    let trimmed = input.trim();
    let lowered = trimmed.to_ascii_lowercase();

    for prefix in ["openalex:", "oa:"] {
        if lowered.starts_with(prefix) {
            return AddSourceIntent::OpenAlexQuery(trimmed[prefix.len()..].trim().to_string());
        }
    }
    for prefix in ["podcast:", "feed:"] {
        if lowered.starts_with(prefix) {
            return AddSourceIntent::PodcastFeed(trimmed[prefix.len()..].trim().to_string());
        }
    }
    for prefix in ["site:", "website:"] {
        if lowered.starts_with(prefix) {
            return AddSourceIntent::WebsitePage(trimmed[prefix.len()..].trim().to_string());
        }
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if trimmed.contains("youtube.com") || trimmed.contains("youtu.be") {
            AddSourceIntent::YouTubeChannel
        } else {
            AddSourceIntent::PodcastFeed(trimmed.to_string())
        }
    } else {
        AddSourceIntent::YouTubeChannel
    }
}

async fn delete_channel_with_search_cleanup(
    state: &AppState,
    channel_id: &str,
) -> Result<bool, String> {
    let video_ids = db::list_video_ids_by_channel(&state.db, channel_id)
        .await
        .map_err(|err| err.to_string())?;

    for video_id in &video_ids {
        for source_kind in [SearchSourceKind::Transcript, SearchSourceKind::Summary] {
            state.fts.delete_source(video_id, source_kind).await?;
        }
    }

    db::delete_channel(&state.db, channel_id)
        .await
        .map_err(|err| err.to_string())
}

async fn source_profile_for_channel(
    store: &db::Store,
    channel: &Channel,
) -> Result<SourceProfileRecord, db::StoreError> {
    if let Some(profile) = db::get_source_profile(store, &channel.id).await? {
        Ok(profile)
    } else {
        let source = crate::models::fallback_source_from_channel(channel);
        let container = crate::models::fallback_container_from_source(&source);
        Ok(SourceProfileRecord {
            source,
            container,
            openalex_query: None,
        })
    }
}

async fn build_snapshot_payload(
    store: &db::Store,
    snapshot: db::ChannelSnapshotData,
) -> Result<crate::models::ChannelSnapshotPayload, db::StoreError> {
    let profile = source_profile_for_channel(store, &snapshot.channel).await?;
    let container = profile.container;
    let source = profile.source;
    let items = snapshot
        .videos
        .iter()
        .map(|video| crate::models::content_item_from_video(video, &source))
        .collect::<Vec<_>>();
    let parts = snapshot
        .videos
        .iter()
        .flat_map(|video| crate::models::content_parts_from_video(video, &source))
        .collect::<Vec<_>>();

    Ok(crate::models::ChannelSnapshotPayload {
        channel_id: snapshot.channel.id.clone(),
        source_id: snapshot.channel.id.clone(),
        container,
        source,
        sync_depth: build_sync_depth_payload(
            &snapshot.channel,
            snapshot.derived_earliest_ready_date,
        ),
        channel_video_count: snapshot.channel_video_count,
        has_more: snapshot.has_more,
        next_offset: snapshot.next_offset,
        videos: snapshot.videos,
        items,
        parts,
    })
}

#[utoipa::path(
    get,
    path = "/api/channels",
    responses(
        (status = 200, description = "Accessible channels", body = [Channel]),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn list_channels(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Do not cache: subscription fields (e.g. `earliest_sync_date`) can change outside the API
    // (S3 ops, migration tools); stale list rows keep the sync boundary input empty.
    let channels = match access_context.user_id.as_deref() {
        Some(user_id) if access_context.auth_state == AuthState::Authenticated => {
            db::list_user_channels_with_virtual_others(&state.db, user_id)
                .await
                .map_err(map_db_err)?
        }
        _ => {
            let mut channels = Vec::new();
            for channel_id in &access_context.allowed_channel_ids {
                if let Some(channel) = db::get_channel(&state.db, channel_id)
                    .await
                    .map_err(map_db_err)?
                {
                    channels.push(channel);
                }
            }
            channels
        }
    };
    Ok(Json(channels))
}

#[utoipa::path(
    get,
    path = "/api/workspace/bootstrap",
    params(WorkspaceBootstrapParams),
    responses(
        (status = 200, description = "Workspace bootstrap payload", body = crate::models::WorkspaceBootstrapPayload),
        (status = 404, description = "Channel not found", body = String),
        (status = 500, description = "Request failed", body = String)
    )
)]
pub async fn workspace_bootstrap(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Query(params): Query<WorkspaceBootstrapParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let video_params = params.video_params();
    // Do not cache: same as `list_channels` / sync depth — subscription and snapshot rows must
    // reflect store writes immediately (including S3 migrations).

    let ai_available = state.summarizer.is_available().await;
    let ai_status = state
        .summarizer
        .indicator_status(state.cloud_cooldown.is_active(), ai_available);
    let channels = match access_context.user_id.as_deref() {
        Some(user_id) if access_context.auth_state == AuthState::Authenticated => {
            db::list_user_channels_with_virtual_others(&state.db, user_id)
                .await
                .map_err(map_db_err)?
        }
        _ => {
            let mut channels = Vec::new();
            for channel_id in &access_context.allowed_channel_ids {
                if let Some(channel) = db::get_channel(&state.db, channel_id)
                    .await
                    .map_err(map_db_err)?
                {
                    channels.push(channel);
                }
            }
            channels
        }
    };
    let selected_channel = params
        .selected_source_id()
        .and_then(|id| channels.iter().find(|channel| channel.id == id))
        .cloned()
        .or_else(|| channels.first().cloned());
    let snapshot = match selected_channel.clone() {
        Some(channel) => {
            let page = db::list_user_scoped_videos_by_channel(
                &state.db,
                access_context.user_id.as_deref(),
                &channel.id,
                &access_context.allowed_channel_ids,
                &access_context.allowed_other_video_ids,
                video_params.limit_or_default(),
                video_params.offset_or_default(),
                video_params.is_short_filter(),
                video_params.acknowledged_filter(),
                video_params.queue_filter(),
            )
            .await
            .map_err(map_db_err)?;
            let page = page.ok_or((StatusCode::NOT_FOUND, "Channel not found".to_string()))?;
            let derived = if channel.id == crate::models::OTHERS_CHANNEL_ID {
                None
            } else {
                db::get_oldest_ready_video_published_at(&state.db, &channel)
                    .await
                    .map_err(map_db_err)?
            };
            Some(db::ChannelSnapshotData {
                channel,
                derived_earliest_ready_date: derived,
                channel_video_count: None,
                has_more: page.has_more,
                next_offset: page.next_offset,
                videos: page.videos,
            })
        }
        None => None,
    };
    let search_status = super::search::load_search_status_payload(&state);
    let mut containers = Vec::with_capacity(channels.len());
    let mut sources = Vec::with_capacity(channels.len());
    for channel in &channels {
        let profile = source_profile_for_channel(&state.db, channel)
            .await
            .map_err(map_db_err)?;
        containers.push(profile.container);
        sources.push(profile.source);
    }

    let payload = crate::models::WorkspaceBootstrapPayload {
        ai_available,
        ai_status,
        containers,
        sources,
        channels,
        selected_source_id: selected_channel.as_ref().map(|channel| channel.id.clone()),
        selected_channel_id: selected_channel.as_ref().map(|channel| channel.id.clone()),
        selected_item_id: params.selected_item_id.clone(),
        snapshot: match snapshot {
            Some(snapshot) => Some(
                build_snapshot_payload(&state.db, snapshot)
                    .await
                    .map_err(map_db_err)?,
            ),
            None => None,
        },
        search_status,
    };

    Ok(Json(payload))
}

#[utoipa::path(
    post,
    path = "/api/channels",
    request_body = AddChannelRequest,
    responses(
        (status = 201, description = "Subscribed source channel", body = Channel),
        (status = 400, description = "Invalid source input", body = String),
        (status = 403, description = "Sign-in required", body = String),
        (status = 502, description = "Upstream source resolution failed", body = String)
    )
)]
pub async fn add_channel(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<AddChannelRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    let input = payload.input.trim().to_string();
    let now = Utc::now();

    match parse_add_source_intent(&input) {
        AddSourceIntent::YouTubeChannel => {
            let resolved = state
                .youtube
                .resolve_feed_source(&input)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let profile = SourceProfileRecord {
                source: resolved.source,
                container: resolved.container,
                openalex_query: None,
            };
            let channel_id = profile.source.id.clone();
            tracing::info!(
                channel_id = %channel_id,
                user_id = %user_id,
                input = %input,
                "resolved youtube source input"
            );

            let channel = Channel {
                id: channel_id.clone(),
                handle: profile.source.handle.clone(),
                name: profile.source.title.clone(),
                thumbnail_url: profile.source.thumbnail_url.clone(),
                added_at: now,
                earliest_sync_date: Some(db::default_earliest_sync_date_floor(now)),
                earliest_sync_date_user_set: false,
            };

            db::insert_channel(&state.db, &channel)
                .await
                .map_err(map_db_err)?;
            db::put_source_profile(&state.db, &profile)
                .await
                .map_err(map_db_err)?;
            db::save_user_channel(&state.db, user_id, &channel)
                .await
                .map_err(map_db_err)?;
            audit::log_channel_subscribe(user_id, &channel, input.len());

            let db_pool = state.db.clone();
            let read_cache = state.read_cache.clone();
            let youtube = state.youtube.clone();
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
        AddSourceIntent::OpenAlexQuery(query) => {
            let structured_query =
                payload
                    .openalex_query
                    .clone()
                    .unwrap_or(crate::models::OpenAlexSavedSearchQuery {
                        natural_language_query: query.clone(),
                        query_text: query.clone(),
                        from_publication_date: None,
                        to_publication_date: None,
                        work_type: None,
                        open_access_only: None,
                        search_scope: crate::models::OpenAlexSearchScope::TitleAndAbstract,
                        sort: crate::models::OpenAlexSort::PublicationDateDesc,
                    });
            let resolved = state
                .openalex
                .resolve_query_source(&structured_query)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let profile = SourceProfileRecord {
                source: resolved.source,
                container: resolved.container,
                openalex_query: Some(structured_query),
            };
            let channel = persist_source_profile_and_channel(&state.db, &profile)
                .await
                .map_err(map_db_err)?;
            if let Err(err) = sync_source_profile(&state, &profile).await {
                let _ = delete_channel_with_search_cleanup(&state, &channel.id).await;
                return Err((StatusCode::BAD_GATEWAY, err));
            }
            db::save_user_channel(
                &state.db,
                user_id,
                &Channel {
                    added_at: now,
                    ..channel.clone()
                },
            )
            .await
            .map_err(map_db_err)?;
            audit::log_channel_subscribe(user_id, &channel, input.len());
            state.read_cache.evict_channel(&channel.id).await;
            Ok((StatusCode::CREATED, Json(channel)))
        }
        AddSourceIntent::PodcastFeed(feed_url) => {
            let profile = match state.podcast_feed.resolve_feed_source(&feed_url).await {
                Ok(resolved) => SourceProfileRecord {
                    source: resolved.source,
                    container: resolved.container,
                    openalex_query: None,
                },
                Err(primary_error) => {
                    let material = state
                        .website
                        .resolve_page(&feed_url)
                        .await
                        .map_err(|_| (StatusCode::BAD_REQUEST, primary_error.to_string()))?;
                    SourceProfileRecord {
                        source: material.source,
                        container: material.container,
                        openalex_query: None,
                    }
                }
            };
            let channel = persist_source_profile_and_channel(&state.db, &profile)
                .await
                .map_err(map_db_err)?;
            if let Err(err) = sync_source_profile(&state, &profile).await {
                let _ = delete_channel_with_search_cleanup(&state, &channel.id).await;
                return Err((StatusCode::BAD_GATEWAY, err));
            }
            db::save_user_channel(
                &state.db,
                user_id,
                &Channel {
                    added_at: now,
                    ..channel.clone()
                },
            )
            .await
            .map_err(map_db_err)?;
            audit::log_channel_subscribe(user_id, &channel, input.len());
            state.read_cache.evict_channel(&channel.id).await;
            Ok((StatusCode::CREATED, Json(channel)))
        }
        AddSourceIntent::WebsitePage(url) => {
            let material = state
                .website
                .resolve_page(&url)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let profile = SourceProfileRecord {
                source: material.source,
                container: material.container,
                openalex_query: None,
            };
            let channel = persist_source_profile_and_channel(&state.db, &profile)
                .await
                .map_err(map_db_err)?;
            if let Err(err) = sync_source_profile(&state, &profile).await {
                let _ = delete_channel_with_search_cleanup(&state, &channel.id).await;
                return Err((StatusCode::BAD_GATEWAY, err));
            }
            db::save_user_channel(
                &state.db,
                user_id,
                &Channel {
                    added_at: now,
                    ..channel.clone()
                },
            )
            .await
            .map_err(map_db_err)?;
            audit::log_channel_subscribe(user_id, &channel, input.len());
            state.read_cache.evict_channel(&channel.id).await;
            Ok((StatusCode::CREATED, Json(channel)))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/openalex/plan",
    request_body = OpenAlexPlanRequest,
    responses(
        (status = 200, description = "Planned OpenAlex query", body = OpenAlexPlanResponse),
        (status = 502, description = "Planner failed", body = String)
    )
)]
pub async fn plan_openalex_query(
    State(state): State<AppState>,
    Json(payload): Json<OpenAlexPlanRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let response: OpenAlexPlanResponse = state
        .openalex_planner
        .plan(&payload.natural_language_query)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error))?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/channels/{id}",
    params(
        ("id" = String, Path, description = "Channel id")
    ),
    responses(
        (status = 200, description = "Channel", body = Channel),
        (status = 404, description = "Channel not found", body = String)
    )
)]
pub async fn get_channel(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(
        require_channel_for_access(&state, &access_context, &id).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/channels/{id}/sync-depth",
    params(
        ("id" = String, Path, description = "Channel id")
    ),
    responses(
        (status = 200, description = "Channel sync depth", body = crate::models::SyncDepthPayload),
        (status = 404, description = "Channel not found", body = String)
    )
)]
pub async fn get_channel_sync_depth(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Do not cache: subscription `earliest_sync_date` can change outside the API (e.g. S3 ops);
    // stale sync-depth misleads the UI until TTL expires on every layer.
    let channel = require_channel_for_access(&state, &access_context, &id).await?;

    let derived = db::get_oldest_ready_video_published_at(&state.db, &channel)
        .await
        .map_err(map_db_err)?;

    let payload = build_sync_depth_payload(&channel, derived);
    Ok(Json(payload))
}

#[utoipa::path(
    get,
    path = "/api/channels/{id}/snapshot",
    params(
        ("id" = String, Path, description = "Channel id"),
        VideoListParams
    ),
    responses(
        (status = 200, description = "Channel snapshot", body = crate::models::ChannelSnapshotPayload),
        (status = 404, description = "Channel not found", body = String)
    )
)]
pub async fn get_channel_snapshot(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(id): Path<String>,
    Query(params): Query<VideoListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let scope = access_context.cache_scope_key();
    let cache_key = ChannelSnapshotCacheKey {
        scope: scope.clone(),
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

    let channel = require_channel_for_access(&state, &access_context, &id).await?;
    let page = db::list_user_scoped_videos_by_channel(
        &state.db,
        access_context.user_id.as_deref(),
        &id,
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
    let derived = if id == crate::models::OTHERS_CHANNEL_ID {
        None
    } else {
        db::get_oldest_ready_video_published_at(&state.db, &channel)
            .await
            .map_err(map_db_err)?
    };
    let payload = build_snapshot_payload(
        &state.db,
        db::ChannelSnapshotData {
            channel,
            derived_earliest_ready_date: derived,
            channel_video_count: None,
            has_more: page.has_more,
            next_offset: page.next_offset,
            videos: page.videos,
        },
    )
    .await
    .map_err(map_db_err)?;
    state
        .read_cache
        .set_channel_snapshot(cache_key, payload.clone())
        .await;
    Ok(Json(payload))
}

#[utoipa::path(
    delete,
    path = "/api/channels/{id}",
    params(
        ("id" = String, Path, description = "Channel id")
    ),
    responses(
        (status = 204, description = "Deleted channel subscription"),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Channel not found", body = String)
    )
)]
pub async fn delete_channel(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    let deleted = db::delete_user_channel_subscription(&state.db, user_id, &id)
        .await
        .map_err(map_db_err)?;

    if deleted {
        audit::log_channel_unsubscribe(user_id, &id);
        state.read_cache.evict_channel(&id).await;
        state.read_cache.evict_channel_list().await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Channel not found".to_string()))
    }
}

#[utoipa::path(
    put,
    path = "/api/channels/{id}",
    params(
        ("id" = String, Path, description = "Channel id")
    ),
    request_body = UpdateChannelRequest,
    responses(
        (status = 200, description = "Updated channel subscription", body = Channel),
        (status = 403, description = "Sign-in required", body = String),
        (status = 404, description = "Channel not found", body = String)
    )
)]
pub async fn update_channel(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let Some(user_id) = access_context.user_id.as_deref() else {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    };
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    let mut channel = db::get_user_channel(&state.db, user_id, &id)
        .await
        .map_err(map_db_err)?
        .ok_or((StatusCode::NOT_FOUND, "Channel not found".to_string()))?;

    let before = channel.clone();

    if let Some(v) = payload.earliest_sync_date {
        channel.earliest_sync_date = Some(v);
    }
    if let Some(v) = payload.earliest_sync_date_user_set {
        channel.earliest_sync_date_user_set = v;
    }

    {
        db::save_user_channel(&state.db, user_id, &channel)
            .await
            .map_err(map_db_err)?;
    }
    audit::log_channel_update(user_id, &id, &before, &channel, &payload);
    state.read_cache.evict_channel(&id).await;
    state.read_cache.evict_channel_list().await;

    Ok(Json(channel))
}

const REFRESH_BACKFILL_BATCH: usize = 50;
const REFRESH_BACKFILL_MAX_ROUNDS: usize = 20;

#[utoipa::path(
    post,
    path = "/api/channels/{id}/refresh",
    params(
        ("id" = String, Path, description = "Channel id")
    ),
    responses(
        (status = 200, description = "Refresh result", body = crate::openapi::VideosAddedResponse),
        (status = 404, description = "Channel not found", body = String),
        (status = 502, description = "Upstream source fetch failed", body = String)
    )
)]
pub async fn refresh_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "refresh requested - queueing latest videos");

    if let Some(profile) = db::get_source_profile(&state.db, &id)
        .await
        .map_err(map_db_err)?
    {
        if profile.source.provider != crate::models::ProviderKind::YouTube {
            let count = sync_source_profile(&state, &profile)
                .await
                .map_err(|err| (StatusCode::BAD_GATEWAY, err))?;
            state.read_cache.evict_channel(&id).await;
            return Ok(Json(serde_json::json!({ "videos_added": count })));
        }
    }

    let earliest_sync_date = require_channel(&state, &id).await?.earliest_sync_date;

    let videos = state.youtube.fetch_videos(&id).await.map_err(map_db_err)?;

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

#[utoipa::path(
    post,
    path = "/api/channels/{id}/backfill",
    params(
        ("id" = String, Path, description = "Channel id"),
        BackfillParams
    ),
    responses(
        (status = 200, description = "Backfill result", body = crate::openapi::ChannelBackfillResponse),
        (status = 404, description = "Channel not found", body = String),
        (status = 502, description = "Upstream source fetch failed", body = String)
    )
)]
pub async fn backfill_channel_videos(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<BackfillParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!(channel_id = %id, "backfill requested");

    if let Some(profile) = db::get_source_profile(&state.db, &id)
        .await
        .map_err(map_db_err)?
    {
        if profile.source.provider != crate::models::ProviderKind::YouTube {
            return Ok(Json(serde_json::json!({
                "videos_added": 0,
                "fetched_count": 0,
                "exhausted": true
            })));
        }
    }

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
        Extension,
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
        security::{AccessContext, AccessRole, AuthState},
        services::{
            ChatService, CloudCooldown, OllamaCore, OpenAlexService, PodcastFeedService,
            SearchService, SummarizerService, SummaryEvaluatorService, TranscriptCooldown,
            TranscriptService, UserActivity, WebsiteService, YouTubeQuotaCooldown, YouTubeService,
        },
        state::AppState,
    };

    async fn test_app_state(db: crate::db::Store) -> AppState {
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
            openalex_planner: Arc::new(crate::services::OpenAlexPlannerService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
            )),
            openalex: Arc::new(OpenAlexService::with_client(Client::new())),
            podcast_feed: Arc::new(PodcastFeedService::with_client(Client::new())),
            website: Arc::new(WebsiteService::with_client(Client::new())),
            transcript: Arc::new(TranscriptService::with_path("/usr/bin/false")),
            tts: None,
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
            input_guardrails: Arc::new(crate::services::InputGuardrailService::new(
                OllamaCore::new("://invalid-url", "qwen3:8b").with_cloud_cooldown(cooldown.clone()),
                Vec::new(),
                Vec::new(),
            )),
            analytics: None,
            active_replies: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            conversation_store_lock: Arc::new(tokio::sync::Mutex::new(())),
            fts: Arc::new(crate::services::FtsIndex::new().await.expect("fts index")),
            anonymous_chat_quota_lock: Arc::new(tokio::sync::Mutex::new(())),
            mobile_auth_handoffs: Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            cloud_cooldown: cooldown,
            youtube_quota_cooldown: Arc::new(YouTubeQuotaCooldown::youtube_quota()),
            transcript_cooldown: Arc::new(TranscriptCooldown::transcript()),
            user_activity: Arc::new(UserActivity::from_env()),
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
                timed_text: None,
            },
        )
        .await
        .unwrap();

        let state = test_app_state(store.clone()).await;
        let materials = list_search_progress_materials(&store).await.unwrap();
        state
            .search_progress
            .initialize_from_materials(&materials, false, false)
            .await;

        let response = workspace_bootstrap(
            State(state),
            Extension(AccessContext {
                user_id: None,
                auth_state: AuthState::Anonymous,
                access_role: AccessRole::Anonymous,
                allowed_channel_ids: vec![channel.id.clone()],
                allowed_other_video_ids: Vec::new(),
            }),
            Query(WorkspaceBootstrapParams::default()),
        )
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
