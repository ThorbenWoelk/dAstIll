use std::collections::HashSet;
use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::db;
use crate::handlers::content;
use crate::models::{AiStatus, ContentStatus, Video};
use crate::services::search::{
    SEARCH_SUMMARY_TARGET_WORDS, SEARCH_TRANSCRIPT_OVERLAP_WORDS, SEARCH_TRANSCRIPT_TARGET_WORDS,
    SearchIndexChunk, SearchSourceKind, build_embedding_input, chunk_summary_content,
    chunk_transcript_content, hash_search_content, vector_to_json,
};
use crate::state::AppState;

const QUEUE_SCAN_LIMIT: usize = 4;
const QUEUE_POLL_INTERVAL: Duration = Duration::from_secs(5);
const CHANNEL_REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);
const CHANNEL_GAP_SCAN_INTERVAL: Duration = Duration::from_secs(10 * 60);
const CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL: usize = 8;
const SUMMARY_EVAL_SCAN_LIMIT: usize = 4;
const SUMMARY_EVAL_POLL_INTERVAL: Duration = Duration::from_secs(7);
const SEARCH_BACKFILL_SCAN_LIMIT: usize = 64;
const SEARCH_INDEX_SCAN_LIMIT: usize = 8;
const SEARCH_RECONCILE_SCAN_LIMIT: usize = 64;
const SEARCH_PRUNE_SCAN_LIMIT: usize = 256;
const SEARCH_INDEX_POLL_INTERVAL: Duration = Duration::from_secs(3);
const SEARCH_RECONCILE_INTERVAL: Duration = Duration::from_secs(60);
const SEARCH_VECTOR_INDEX_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const MAX_DISTILLATION_RETRIES: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QueueTask {
    Transcript,
    Summary,
    Skip,
}

fn next_queue_task(video: &Video) -> QueueTask {
    if video.retry_count >= MAX_DISTILLATION_RETRIES {
        return QueueTask::Skip;
    }

    match video.transcript_status {
        ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed => {
            QueueTask::Transcript
        }
        ContentStatus::Ready => match video.summary_status {
            ContentStatus::Pending | ContentStatus::Loading | ContentStatus::Failed => {
                QueueTask::Summary
            }
            ContentStatus::Ready => QueueTask::Skip,
        },
    }
}

fn should_queue_summary_auto_regeneration(quality_score: u8, auto_regen_attempts: u8) -> bool {
    quality_score < content::MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE
        && auto_regen_attempts < content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS
}

fn should_run_summary_evaluation(evaluator_status: AiStatus, evaluator_model: &str) -> bool {
    match evaluator_status {
        AiStatus::Cloud => true,
        AiStatus::LocalOnly => !crate::services::is_cloud_model(evaluator_model),
        AiStatus::Offline => false,
    }
}

fn chunk_material(material: &db::SearchMaterial) -> Vec<crate::services::search::ChunkDraft> {
    match material.source_kind {
        SearchSourceKind::Transcript => chunk_transcript_content(
            &material.content,
            SEARCH_TRANSCRIPT_TARGET_WORDS,
            SEARCH_TRANSCRIPT_OVERLAP_WORDS,
        ),
        SearchSourceKind::Summary => {
            chunk_summary_content(&material.content, SEARCH_SUMMARY_TARGET_WORDS)
        }
    }
}

async fn backfill_search_sources(state: &AppState) {
    let _projection_guard = state.search_projection_lock.read().await;
    let materials = {
        let conn = state.db.connect();
        db::list_search_backfill_materials(&conn, SEARCH_BACKFILL_SCAN_LIMIT)
            .await
            .map_err(|err| err.to_string())
    };

    let materials = match materials {
        Ok(materials) => materials,
        Err(err) => {
            tracing::error!(error = %err, "search backfill failed to load existing materials");
            return;
        }
    };

    let discovered_count = materials.len();
    let mut queued = 0usize;
    let mut failed = 0usize;
    for material in materials {
        let content_hash = hash_search_content(&material.content);
        let conn = state.db.connect();
        if let Err(err) = db::mark_search_source_pending(
            &conn,
            &material.video_id,
            material.source_kind,
            &content_hash,
        )
        .await
        {
            tracing::error!(
                video_id = %material.video_id,
                source_kind = material.source_kind.as_str(),
                error = %err,
                "search backfill failed to queue source"
            );
            failed += 1;
            continue;
        }
        queued += 1;
    }

    if discovered_count > 0 || failed > 0 {
        tracing::info!(
            batch_limit = SEARCH_BACKFILL_SCAN_LIMIT,
            discovered_count,
            queued_count = queued,
            failed_count = failed,
            "search backfill round complete"
        );
    }
}

async fn reconcile_search_sources(state: &AppState) {
    let _projection_guard = state.search_projection_lock.read().await;
    let materials = {
        let conn = state.db.connect();
        db::list_search_reconciliation_materials(&conn, SEARCH_RECONCILE_SCAN_LIMIT)
            .await
            .map_err(|err| err.to_string())
    };

    let materials = match materials {
        Ok(materials) => materials,
        Err(err) => {
            tracing::error!(error = %err, "search reconcile failed to load materials");
            return;
        }
    };

    let inspected_count = materials.len();
    let mut refreshed_count = 0usize;
    let mut failed_count = 0usize;
    for material in materials {
        let content_hash = hash_search_content(&material.content);
        let conn = state.db.connect();
        let state_row =
            db::get_search_source_state(&conn, &material.video_id, material.source_kind).await;
        let state_row = match state_row {
            Ok(value) => value,
            Err(err) => {
                tracing::error!(
                    video_id = %material.video_id,
                    source_kind = material.source_kind.as_str(),
                    error = %err,
                    "search reconcile failed to inspect source state"
                );
                failed_count += 1;
                continue;
            }
        };

        let Some(state_row) = state_row else {
            continue;
        };

        let needs_refresh = state_row.content_hash != content_hash
            || state_row.index_status == "failed"
            || (state.search.semantic_enabled()
                && state_row.embedding_model.as_deref() != state.search.model());

        if needs_refresh {
            if let Err(err) = db::mark_search_source_pending(
                &conn,
                &material.video_id,
                material.source_kind,
                &content_hash,
            )
            .await
            {
                tracing::error!(
                    video_id = %material.video_id,
                    source_kind = material.source_kind.as_str(),
                    error = %err,
                    "search reconcile failed to mark source pending"
                );
                failed_count += 1;
            } else {
                refreshed_count += 1;
            }
        }
    }

    if refreshed_count > 0 || failed_count > 0 {
        tracing::info!(
            inspected_count,
            refreshed_count,
            failed_count,
            "search reconcile round complete"
        );
    }
}

async fn process_pending_search_sources(state: &AppState) {
    let semantic_enabled = state.search.semantic_enabled();
    if semantic_enabled && !state.search.is_available().await {
        tracing::warn!(
            "search index worker skipped - Ollama embedding model not found in /api/tags"
        );
        return;
    }
    let _projection_guard = state.search_projection_lock.read().await;

    let pending_sources = {
        let conn = state.db.connect();
        db::list_pending_search_sources(&conn, SEARCH_INDEX_SCAN_LIMIT)
            .await
            .map_err(|err| err.to_string())
    };

    let pending_sources = match pending_sources {
        Ok(pending_sources) => pending_sources,
        Err(err) => {
            tracing::error!(error = %err, "search index worker failed to load pending sources");
            return;
        }
    };

    let discovered_count = pending_sources.len();
    let mut claimed_count = 0usize;
    let mut indexed_count = 0usize;
    let mut cleared_count = 0usize;
    let mut requeued_count = 0usize;
    let mut embedded_chunk_count = 0usize;
    let mut failed_count = 0usize;

    // Phase 1: Claim sources and load materials, collecting all embedding work.
    struct PreparedSource {
        video_id: String,
        source_kind: SearchSourceKind,
        content_hash: String,
        drafts: Vec<crate::services::search::ChunkDraft>,
        embedding_inputs: Vec<String>,
    }

    let mut prepared = Vec::new();
    for source in pending_sources {
        let conn = state.db.connect();
        let claimed = match db::mark_search_source_indexing(
            &conn,
            &source.video_id,
            source.source_kind,
            &source.content_hash,
        )
        .await
        {
            Ok(claimed) => claimed,
            Err(err) => {
                tracing::error!(
                    video_id = %source.video_id,
                    source_kind = source.source_kind.as_str(),
                    error = %err,
                    "search index worker failed to claim source"
                );
                failed_count += 1;
                continue;
            }
        };

        if !claimed {
            continue;
        }
        claimed_count += 1;

        let material =
            match db::load_search_material(&conn, &source.video_id, source.source_kind).await {
                Ok(material) => material,
                Err(err) => {
                    let _ = db::mark_search_source_failed(
                        &conn,
                        &source.video_id,
                        source.source_kind,
                        &source.content_hash,
                        &err.to_string(),
                    )
                    .await;
                    failed_count += 1;
                    continue;
                }
            };

        let Some(material) = material else {
            let _ = db::clear_search_source(&conn, &source.video_id, source.source_kind).await;
            cleared_count += 1;
            continue;
        };

        let current_hash = hash_search_content(&material.content);
        if current_hash != source.content_hash {
            let _ = db::mark_search_source_pending(
                &conn,
                &source.video_id,
                source.source_kind,
                &current_hash,
            )
            .await;
            requeued_count += 1;
            continue;
        }

        let drafts = chunk_material(&material);
        if drafts.is_empty() {
            let _ = db::clear_search_source(&conn, &source.video_id, source.source_kind).await;
            cleared_count += 1;
            continue;
        }

        let embedding_inputs = if semantic_enabled {
            drafts
                .iter()
                .map(|draft| {
                    build_embedding_input(
                        &material.video_title,
                        &material.channel_name,
                        draft.source_kind,
                        draft.section_title.as_deref(),
                        &draft.text,
                    )
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        prepared.push(PreparedSource {
            video_id: source.video_id,
            source_kind: source.source_kind,
            content_hash: source.content_hash,
            drafts,
            embedding_inputs,
        });
    }

    if prepared.is_empty() {
        if discovered_count > 0 || failed_count > 0 {
            tracing::info!(
                batch_limit = SEARCH_INDEX_SCAN_LIMIT,
                semantic_enabled,
                embedding_model = %state.search.model_label(),
                discovered_count,
                claimed_count,
                indexed_count,
                cleared_count,
                requeued_count,
                embedded_chunk_count,
                failed_count,
                "search indexing round complete"
            );
        }
        return;
    }

    if !semantic_enabled {
        for source in prepared {
            let chunk_count = source.drafts.len();
            let chunks = source
                .drafts
                .into_iter()
                .enumerate()
                .map(|(index, draft)| SearchIndexChunk {
                    chunk_index: index,
                    section_title: draft.section_title,
                    chunk_text: draft.text,
                    embedding_json: None,
                    token_count: draft.word_count,
                })
                .collect::<Vec<_>>();

            let write_start = std::time::Instant::now();
            let conn = state.db.connect();
            match db::replace_search_chunks(
                &conn,
                &source.video_id,
                source.source_kind,
                &source.content_hash,
                None,
                &chunks,
            )
            .await
            {
                Ok(stored) => {
                    if stored {
                        indexed_count += 1;
                    }
                }
                Err(err) => {
                    tracing::error!(
                        video_id = %source.video_id,
                        source_kind = source.source_kind.as_str(),
                        chunk_count,
                        elapsed_ms = write_start.elapsed().as_millis() as u64,
                        error = %err,
                        "search index: FTS-only source write failed"
                    );
                    let _ = db::mark_search_source_failed(
                        &conn,
                        &source.video_id,
                        source.source_kind,
                        &source.content_hash,
                        &err.to_string(),
                    )
                    .await;
                    failed_count += 1;
                }
            }
        }

        if discovered_count > 0 || failed_count > 0 {
            tracing::info!(
                batch_limit = SEARCH_INDEX_SCAN_LIMIT,
                semantic_enabled,
                embedding_model = %state.search.model_label(),
                discovered_count,
                claimed_count,
                indexed_count,
                cleared_count,
                requeued_count,
                embedded_chunk_count,
                failed_count,
                "search indexing round complete"
            );
        }
        return;
    }

    // Phase 2: Embed all chunks across claimed sources. The search service
    // sub-batches requests so large local Ollama payloads do not time out.
    let all_inputs: Vec<String> = prepared
        .iter()
        .flat_map(|source| source.embedding_inputs.iter().cloned())
        .collect();

    let all_embeddings = match state.search.embed_texts(&all_inputs).await {
        Ok(embeddings) => embeddings,
        Err(err) => {
            // Mark all claimed sources as failed.
            for source in &prepared {
                let conn = state.db.connect();
                let _ = db::mark_search_source_failed(
                    &conn,
                    &source.video_id,
                    source.source_kind,
                    &source.content_hash,
                    &err.to_string(),
                )
                .await;
            }
            tracing::error!(
                error = %err,
                sources = prepared.len(),
                chunks = all_inputs.len(),
                failed_count = failed_count + prepared.len(),
                "search indexing embed batch failed"
            );
            return;
        }
    };
    embedded_chunk_count = all_embeddings.len();

    // Phase 3: Distribute embeddings back to sources and write to DB.
    let mut embedding_offset = 0usize;
    for source in prepared {
        let chunk_count = source.drafts.len();
        let source_embeddings = &all_embeddings[embedding_offset..embedding_offset + chunk_count];
        embedding_offset += chunk_count;

        let chunks = source
            .drafts
            .into_iter()
            .zip(source_embeddings.iter())
            .enumerate()
            .map(|(index, (draft, embedding))| SearchIndexChunk {
                chunk_index: index,
                section_title: draft.section_title,
                chunk_text: draft.text,
                embedding_json: Some(vector_to_json(embedding)),
                token_count: draft.word_count,
            })
            .collect::<Vec<_>>();

        let write_start = std::time::Instant::now();
        let conn = state.db.connect();
        match db::replace_search_chunks(
            &conn,
            &source.video_id,
            source.source_kind,
            &source.content_hash,
            state.search.model(),
            &chunks,
        )
        .await
        {
            Ok(stored) => {
                if stored {
                    indexed_count += 1;
                }
            }
            Err(err) => {
                tracing::error!(
                    video_id = %source.video_id,
                    source_kind = source.source_kind.as_str(),
                    chunk_count,
                    elapsed_ms = write_start.elapsed().as_millis() as u64,
                    error = %err,
                    "search index: source write failed"
                );
                let _ = db::mark_search_source_failed(
                    &conn,
                    &source.video_id,
                    source.source_kind,
                    &source.content_hash,
                    &err.to_string(),
                )
                .await;
                failed_count += 1;
            }
        }
    }

    if discovered_count > 0 || failed_count > 0 {
        tracing::info!(
            batch_limit = SEARCH_INDEX_SCAN_LIMIT,
            semantic_enabled,
            embedding_model = %state.search.model_label(),
            discovered_count,
            claimed_count,
            indexed_count,
            cleared_count,
            requeued_count,
            embedded_chunk_count,
            failed_count,
            "search indexing round complete"
        );
    }
}

async fn prune_stale_search_rows(state: &AppState) {
    let _projection_guard = state.search_projection_lock.read().await;
    let conn = state.db.connect();
    match db::prune_stale_search_rows(&conn, SEARCH_PRUNE_SCAN_LIMIT).await {
        Ok(pruned_count) if pruned_count > 0 => {
            tracing::info!(pruned_count, "search prune round complete");
        }
        Ok(_) => {}
        Err(err) => {
            tracing::error!(error = %err, "search prune failed");
        }
    }
}

async fn maybe_ensure_vector_index(state: &AppState, last_attempt: &mut Option<Instant>) {
    if !state.search_auto_create_vector_index || !state.search.semantic_enabled() {
        return;
    }
    let _projection_guard = state.search_projection_lock.read().await;

    if last_attempt
        .as_ref()
        .is_some_and(|instant| instant.elapsed() < SEARCH_VECTOR_INDEX_RETRY_INTERVAL)
    {
        return;
    }

    let conn = state.db.connect();
    let counts = match db::get_search_source_counts(&conn).await {
        Ok(counts) => counts,
        Err(err) => {
            tracing::error!(error = %err, "search vector index check failed to load counts");
            return;
        }
    };

    if counts.ready == 0 || counts.pending > 0 || counts.indexing > 0 {
        return;
    }

    match db::has_vector_index(&conn).await {
        Ok(true) => return,
        Ok(false) => {}
        Err(err) => {
            tracing::error!(error = %err, "search vector index check failed");
            return;
        }
    }

    *last_attempt = Some(Instant::now());
    tracing::info!(
        ready_sources = counts.ready,
        "search vector index build starting"
    );
    match db::ensure_vector_index(&conn).await {
        Ok(()) => tracing::info!("search vector index build complete"),
        Err(err) => {
            tracing::error!(error = %err, "search vector index build failed");
        }
    }
}

pub fn spawn_queue_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            poll_interval_secs = QUEUE_POLL_INTERVAL.as_secs(),
            "queue worker started"
        );

        loop {
            let queue = {
                let conn = state.db.connect();
                db::list_videos_for_queue_processing(
                    &conn,
                    QUEUE_SCAN_LIMIT,
                    MAX_DISTILLATION_RETRIES,
                )
                .await
                .map_err(|err| err.to_string())
            };

            let queue = match queue {
                Ok(videos) => videos,
                Err(err) => {
                    tracing::error!(error = %err, "queue worker failed to load queue");
                    sleep(QUEUE_POLL_INTERVAL).await;
                    continue;
                }
            };

            for video in queue {
                let task = next_queue_task(&video);

                // Fast-path skip if transcript rate limits apply to avoid log spam
                if task == QueueTask::Transcript && state.transcript_cooldown.is_active() {
                    continue;
                }

                tracing::info!(video_id = %video.id, "queue worker processing video");
                let result = match task {
                    QueueTask::Transcript => {
                        tracing::info!(video_id = %video.id, "queue worker ensuring transcript");
                        content::ensure_transcript(&state, &video.id)
                            .await
                            .map(|_| ())
                    }
                    QueueTask::Summary => {
                        tracing::info!(video_id = %video.id, "queue worker ensuring summary");
                        content::ensure_summary(&state, &video.id).await.map(|_| ())
                    }
                    QueueTask::Skip => {
                        tracing::debug!(video_id = %video.id, "queue worker skipping video");
                        Ok(())
                    }
                };

                if let Err((status, message)) = result {
                    // Only log as warning/increment retry if it's not a quota/rate limit error we know about
                    if status == axum::http::StatusCode::TOO_MANY_REQUESTS {
                        tracing::debug!(
                            video_id = %video.id,
                            "queue worker paused for video due to rate limits"
                        );
                    } else {
                        tracing::warn!(
                            video_id = %video.id,
                            http_status = %status,
                            error = %message,
                            "queue worker failed to process video"
                        );
                        let conn = state.db.connect();
                        let _ = db::increment_video_retry_count(&conn, &video.id).await;
                    }
                } else {
                    let conn = state.db.connect();
                    let _ = db::reset_video_retry_count(&conn, &video.id).await;
                }
            }

            sleep(QUEUE_POLL_INTERVAL).await;
        }
    });
}

/// Refresh all channels by fetching their RSS feeds and inserting new videos.
async fn refresh_all_channels(state: &AppState) {
    let channels = {
        let conn = state.db.connect();
        db::list_channels(&conn)
            .await
            .map_err(|err| err.to_string())
    };

    let channels = match channels {
        Ok(list) => list,
        Err(err) => {
            tracing::error!(error = %err, "refresh worker failed to list channels");
            return;
        }
    };

    if channels.is_empty() {
        return;
    }

    tracing::info!(channel_count = channels.len(), "refreshing all channels");

    for (i, channel) in channels.iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_secs(1)).await;
        }
        match state.youtube.fetch_videos(&channel.id).await {
            Ok(videos) => {
                let conn = state.db.connect();
                let n = db::bulk_insert_videos(&conn, videos).await.unwrap_or(0);
                if n > 0 {
                    tracing::info!(
                        channel_id = %channel.id,
                        new_videos = n,
                        "refresh worker found new videos"
                    );
                }
            }
            Err(err) => {
                tracing::warn!(
                    channel_id = %channel.id,
                    error = %err,
                    "refresh worker failed to fetch videos"
                );
            }
        }
    }
}

pub fn spawn_refresh_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            interval_secs = CHANNEL_REFRESH_INTERVAL.as_secs(),
            "channel refresh worker started"
        );

        // Run an initial refresh at startup so new videos appear immediately.
        refresh_all_channels(&state).await;

        loop {
            sleep(CHANNEL_REFRESH_INTERVAL).await;
            refresh_all_channels(&state).await;
        }
    });
}

async fn fill_channel_gaps(
    state: &AppState,
    channel_id: &str,
    limit: usize,
    until: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<usize, String> {
    let known_video_ids = {
        let conn = state.db.connect();
        db::list_video_ids_by_channel(&conn, channel_id)
            .await
            .map_err(|err| err.to_string())?
            .into_iter()
            .collect::<HashSet<_>>()
    };

    let (videos, _exhausted) = state
        .youtube
        .fetch_videos_backfill_missing(channel_id, &known_video_ids, limit, until)
        .await
        .map_err(|err| err.to_string())?;

    let conn = state.db.connect();
    let inserted = db::bulk_insert_videos(&conn, videos)
        .await
        .map_err(|err| err.to_string())?;
    Ok(inserted)
}

async fn scan_all_channels_for_gaps(state: &AppState) {
    if state.youtube_quota_cooldown.is_active() {
        tracing::debug!("skipping gap scan worker - youtube quota cooldown active");
        return;
    }

    let channels = {
        let conn = state.db.connect();
        db::list_channels(&conn)
            .await
            .map_err(|err| err.to_string())
    };

    let channels = match channels {
        Ok(list) => list,
        Err(err) => {
            tracing::error!(error = %err, "gap scan worker failed to list channels");
            return;
        }
    };

    if channels.is_empty() {
        return;
    }

    tracing::info!(
        channel_count = channels.len(),
        per_channel_limit = CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
        "gap scan worker scanning channels"
    );

    for (i, channel) in channels.into_iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_secs(1)).await;
        }
        match fill_channel_gaps(
            state,
            &channel.id,
            CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
            channel.earliest_sync_date,
        )
        .await
        {
            Ok(inserted) if inserted > 0 => {
                tracing::info!(
                    channel_id = %channel.id,
                    inserted,
                    "gap scan worker inserted missing videos"
                );
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(
                    channel_id = %channel.id,
                    error = %err,
                    "gap scan worker failed for channel"
                );
            }
        }
    }
}

pub fn spawn_gap_scan_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            interval_secs = CHANNEL_GAP_SCAN_INTERVAL.as_secs(),
            per_channel_limit = CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
            "channel gap scan worker started"
        );

        scan_all_channels_for_gaps(&state).await;

        loop {
            sleep(CHANNEL_GAP_SCAN_INTERVAL).await;
            scan_all_channels_for_gaps(&state).await;
        }
    });
}

pub fn spawn_summary_evaluation_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            poll_interval_secs = SUMMARY_EVAL_POLL_INTERVAL.as_secs(),
            model = %state.summary_evaluator.model(),
            "summary evaluation worker started"
        );

        loop {
            let queue = {
                let conn = state.db.connect();
                db::list_summaries_pending_quality_eval(&conn, SUMMARY_EVAL_SCAN_LIMIT)
                    .await
                    .map_err(|err| err.to_string())
            };

            let queue = match queue {
                Ok(rows) => rows,
                Err(err) => {
                    tracing::error!(error = %err, "summary evaluation worker failed to load queue");
                    sleep(SUMMARY_EVAL_POLL_INTERVAL).await;
                    continue;
                }
            };

            if queue.is_empty() {
                sleep(SUMMARY_EVAL_POLL_INTERVAL).await;
                continue;
            }

            let evaluator_available = state.summary_evaluator.is_available().await;
            let evaluator_status = state
                .summary_evaluator
                .indicator_status(state.cloud_cooldown.is_active(), evaluator_available);

            if !should_run_summary_evaluation(evaluator_status, state.summary_evaluator.model()) {
                tracing::debug!(
                    evaluator_status = ?evaluator_status,
                    "summary evaluation paused - evaluator unavailable or preserving local capacity"
                );
                // Back off longer when evaluator is offline to avoid log spam
                sleep(Duration::from_secs(60)).await;
                continue;
            }

            for job in queue {
                tracing::info!(video_id = %job.video_id, "summary evaluation worker processing video");
                let evaluation = state
                    .summary_evaluator
                    .evaluate(&job.transcript_text, &job.summary_content, &job.video_title)
                    .await;

                match evaluation {
                    Ok(result) => {
                        let conn = state.db.connect();
                        let _ = db::update_summary_quality(
                            &conn,
                            &job.video_id,
                            Some(result.quality_score),
                            result.quality_note.as_deref(),
                            result.quality_model_used.as_deref(),
                        )
                        .await;

                        if let Ok(auto_regen_attempts) =
                            db::get_summary_auto_regen_attempts(&conn, &job.video_id).await
                        {
                            if should_queue_summary_auto_regeneration(
                                result.quality_score,
                                auto_regen_attempts,
                            ) {
                                if let Err(err) = db::update_video_summary_status(
                                    &conn,
                                    &job.video_id,
                                    ContentStatus::Pending,
                                )
                                .await
                                {
                                    tracing::warn!(
                                        video_id = %job.video_id,
                                        error = %err,
                                        "failed to queue low-quality summary regeneration"
                                    );
                                } else {
                                    tracing::info!(
                                        video_id = %job.video_id,
                                        score = result.quality_score,
                                        attempts = auto_regen_attempts,
                                        threshold = content::MIN_SUMMARY_QUALITY_SCORE_FOR_ACCEPTANCE,
                                        max_attempts = content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS,
                                        "queued summary for automatic regeneration"
                                    );
                                }
                            }
                        }
                    }
                    Err(ref err)
                        if matches!(
                            err,
                            crate::services::summary_evaluator::SummaryEvaluatorError::NotAvailable
                        ) =>
                    {
                        tracing::debug!(
                            video_id = %job.video_id,
                            "summary evaluation deferred - evaluator not available"
                        );
                        // Leave quality_score/quality_note NULL so the job is retried later
                    }
                    Err(err) => {
                        tracing::warn!(
                            video_id = %job.video_id,
                            error = %err,
                            "summary evaluation failed"
                        );
                        // Permanent failure - mark with note but no score so it can be retried
                    }
                }
            }

            sleep(SUMMARY_EVAL_POLL_INTERVAL).await;
        }
    });
}

pub fn spawn_search_index_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            backfill_scan_limit = SEARCH_BACKFILL_SCAN_LIMIT,
            index_scan_limit = SEARCH_INDEX_SCAN_LIMIT,
            poll_interval_secs = SEARCH_INDEX_POLL_INTERVAL.as_secs(),
            reconcile_interval_secs = SEARCH_RECONCILE_INTERVAL.as_secs(),
            vector_index_retry_interval_secs = SEARCH_VECTOR_INDEX_RETRY_INTERVAL.as_secs(),
            auto_create_vector_index = state.search_auto_create_vector_index,
            semantic_enabled = state.search.semantic_enabled(),
            model = %state.search.model_label(),
            "search index worker started"
        );

        backfill_search_sources(&state).await;
        process_pending_search_sources(&state).await;
        reconcile_search_sources(&state).await;
        prune_stale_search_rows(&state).await;
        let mut last_vector_index_attempt = None;
        maybe_ensure_vector_index(&state, &mut last_vector_index_attempt).await;
        let mut ticks_since_reconcile = 0usize;

        loop {
            backfill_search_sources(&state).await;
            process_pending_search_sources(&state).await;
            prune_stale_search_rows(&state).await;
            maybe_ensure_vector_index(&state, &mut last_vector_index_attempt).await;
            ticks_since_reconcile += 1;

            if ticks_since_reconcile * SEARCH_INDEX_POLL_INTERVAL.as_secs() as usize
                >= SEARCH_RECONCILE_INTERVAL.as_secs() as usize
            {
                reconcile_search_sources(&state).await;
                maybe_ensure_vector_index(&state, &mut last_vector_index_attempt).await;
                ticks_since_reconcile = 0;
            }

            sleep(SEARCH_INDEX_POLL_INTERVAL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{
        QueueTask, next_queue_task, should_queue_summary_auto_regeneration,
        should_run_summary_evaluation,
    };
    use crate::models::{AiStatus, ContentStatus, Video};

    fn video_with_statuses(
        transcript_status: ContentStatus,
        summary_status: ContentStatus,
    ) -> Video {
        Video {
            id: "video".to_string(),
            channel_id: "channel".to_string(),
            title: "Title".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status,
            summary_status,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }

    #[test]
    fn next_queue_task_prioritizes_transcript_when_not_ready() {
        let video = video_with_statuses(ContentStatus::Pending, ContentStatus::Ready);
        assert_eq!(next_queue_task(&video), QueueTask::Transcript);

        let loading_video = video_with_statuses(ContentStatus::Loading, ContentStatus::Pending);
        assert_eq!(next_queue_task(&loading_video), QueueTask::Transcript);
    }

    #[test]
    fn next_queue_task_summarizes_only_after_transcript_ready() {
        let video = video_with_statuses(ContentStatus::Ready, ContentStatus::Pending);
        assert_eq!(next_queue_task(&video), QueueTask::Summary);

        let loading_summary = video_with_statuses(ContentStatus::Ready, ContentStatus::Loading);
        assert_eq!(next_queue_task(&loading_summary), QueueTask::Summary);
    }

    #[test]
    fn next_queue_task_retries_failed_rows() {
        let failed_transcript = video_with_statuses(ContentStatus::Failed, ContentStatus::Pending);
        assert_eq!(next_queue_task(&failed_transcript), QueueTask::Transcript);

        let failed_summary = video_with_statuses(ContentStatus::Ready, ContentStatus::Failed);
        assert_eq!(next_queue_task(&failed_summary), QueueTask::Summary);
    }

    #[test]
    fn next_queue_task_skips_complete_rows() {
        let done = video_with_statuses(ContentStatus::Ready, ContentStatus::Ready);
        assert_eq!(next_queue_task(&done), QueueTask::Skip);
    }

    #[test]
    fn should_queue_summary_auto_regeneration_only_for_low_scores_with_remaining_attempts() {
        assert!(should_queue_summary_auto_regeneration(6, 0));
        assert!(should_queue_summary_auto_regeneration(0, 1));
        assert!(!should_queue_summary_auto_regeneration(7, 0));
        assert!(!should_queue_summary_auto_regeneration(9, 0));
        assert!(!should_queue_summary_auto_regeneration(6, 2));
    }

    #[test]
    fn summary_evaluation_runs_only_when_it_wont_consume_local_fallback_capacity() {
        assert!(should_run_summary_evaluation(
            AiStatus::Cloud,
            "qwen3.5:397b-cloud"
        ));
        assert!(!should_run_summary_evaluation(
            AiStatus::LocalOnly,
            "qwen3.5:397b-cloud"
        ));
        assert!(should_run_summary_evaluation(
            AiStatus::LocalOnly,
            "qwen3:8b"
        ));
        assert!(!should_run_summary_evaluation(
            AiStatus::Offline,
            "qwen3.5:397b-cloud"
        ));
    }
}
