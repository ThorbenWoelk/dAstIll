use std::time::Instant;
use tracing::Instrument;

use crate::{
    db,
    search_progress::SearchProgressSourceStatus,
    services::search::{
        SEARCH_SUMMARY_TARGET_WORDS, SEARCH_TRANSCRIPT_OVERLAP_WORDS,
        SEARCH_TRANSCRIPT_TARGET_WORDS, SearchIndexChunk, SearchSourceKind, build_embedding_input,
        chunk_summary_content, chunk_transcript_content, hash_search_content, vector_to_json,
    },
    state::AppState,
};

use super::{
    PollBackoffState, SEARCH_BACKFILL_SCAN_LIMIT, SEARCH_INDEX_POLL_BACKOFF,
    SEARCH_INDEX_SCAN_LIMIT, SEARCH_PRUNE_SCAN_LIMIT, SEARCH_RECONCILE_INTERVAL,
    SEARCH_RECONCILE_SCAN_LIMIT, SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD,
    SEARCH_VECTOR_INDEX_RETRY_INTERVAL, sleep_with_backoff,
};
use super::{
    SEARCH_INDEX_IDLE_POLL_INTERVAL, SEARCH_INDEX_IDLE_POLL_MAX_INTERVAL,
    SEARCH_INDEX_POLL_INTERVAL,
};

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

async fn backfill_search_sources(state: &AppState) -> bool {
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
            return false;
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
        state
            .search_progress
            .upsert_material(&material, SearchProgressSourceStatus::Pending, 0)
            .await;
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

    discovered_count > 0 || queued > 0 || failed > 0
}

async fn reconcile_search_sources(state: &AppState) -> bool {
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
            return false;
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
                state
                    .search_progress
                    .upsert_material(&material, SearchProgressSourceStatus::Pending, 0)
                    .await;
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

    refreshed_count > 0 || failed_count > 0
}

async fn process_pending_search_sources(state: &AppState) -> bool {
    let span = logfire::span!(
        "worker.search_index.process_pending",
        batch_limit = SEARCH_INDEX_SCAN_LIMIT,
        semantic_enabled = state.search.semantic_enabled(),
        embedding_model = state.search.model_label().to_string(),
    );

    async move {
        let semantic_enabled = state.search.semantic_enabled();
        let semantic_available = if semantic_enabled {
            state.search.is_available().await
        } else {
            false
        };
        state
            .search_progress
            .set_semantic_available(semantic_available);

        if semantic_enabled && !semantic_available {
            tracing::warn!(
                "search index worker skipped - Ollama embedding model not found in /api/tags"
            );
            return false;
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
                return false;
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
                        state
                            .search_progress
                            .set_source_status(
                                &source.video_id,
                                source.source_kind,
                                SearchProgressSourceStatus::Failed,
                            )
                            .await;
                        failed_count += 1;
                        continue;
                    }
                };

            let Some(material) = material else {
                let _ = db::clear_search_source(&conn, &source.video_id, source.source_kind).await;
                state
                    .search_progress
                    .remove_source(&source.video_id, source.source_kind)
                    .await;
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
                state
                    .search_progress
                    .upsert_material(&material, SearchProgressSourceStatus::Pending, 0)
                    .await;
                requeued_count += 1;
                continue;
            }

            let drafts = chunk_material(&material);
            if drafts.is_empty() {
                let _ = db::clear_search_source(&conn, &source.video_id, source.source_kind).await;
                state
                    .search_progress
                    .remove_source(&source.video_id, source.source_kind)
                    .await;
                cleared_count += 1;
                continue;
            }

            state
                .search_progress
                .upsert_source(
                    &source.video_id,
                    source.source_kind,
                    SearchProgressSourceStatus::Indexing,
                    drafts.len(),
                    0,
                )
                .await;

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
            return discovered_count > 0 || failed_count > 0;
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
                            state
                                .search_progress
                                .upsert_source(
                                    &source.video_id,
                                    source.source_kind,
                                    SearchProgressSourceStatus::Ready,
                                    chunk_count,
                                    0,
                                )
                                .await;
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
                        state
                            .search_progress
                            .set_source_status(
                                &source.video_id,
                                source.source_kind,
                                SearchProgressSourceStatus::Failed,
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
            return discovered_count > 0
                || indexed_count > 0
                || cleared_count > 0
                || requeued_count > 0
                || embedded_chunk_count > 0
                || failed_count > 0;
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
                    state
                        .search_progress
                        .set_source_status(
                            &source.video_id,
                            source.source_kind,
                            SearchProgressSourceStatus::Failed,
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
                return true;
            }
        };
        embedded_chunk_count = all_embeddings.len();

        // Phase 3: Distribute embeddings back to sources and write to DB.
        let mut embedding_offset = 0usize;
        for source in prepared {
            let chunk_count = source.drafts.len();
            let source_embeddings =
                &all_embeddings[embedding_offset..embedding_offset + chunk_count];
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
                        state
                            .search_progress
                            .upsert_source(
                                &source.video_id,
                                source.source_kind,
                                SearchProgressSourceStatus::Ready,
                                chunk_count,
                                chunk_count,
                            )
                            .await;
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
                    state
                        .search_progress
                        .set_source_status(
                            &source.video_id,
                            source.source_kind,
                            SearchProgressSourceStatus::Failed,
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

        discovered_count > 0
            || indexed_count > 0
            || cleared_count > 0
            || requeued_count > 0
            || embedded_chunk_count > 0
            || failed_count > 0
    }
    .instrument(span)
    .await
}

async fn prune_stale_search_rows(state: &AppState) -> bool {
    let _projection_guard = state.search_projection_lock.read().await;
    let conn = state.db.connect();
    match db::prune_stale_search_rows(&conn, SEARCH_PRUNE_SCAN_LIMIT).await {
        Ok(pruned_count) if pruned_count > 0 => {
            tracing::info!(pruned_count, "search prune round complete");
            true
        }
        Ok(_) => false,
        Err(err) => {
            tracing::error!(error = %err, "search prune failed");
            false
        }
    }
}

pub(super) fn should_build_vector_index(counts: &db::SearchSourceCounts) -> bool {
    counts.ready > 0
        && counts.pending.saturating_add(counts.indexing)
            <= SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD
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

    if !should_build_vector_index(&counts) {
        return;
    }

    match db::has_vector_index(&conn).await {
        Ok(true) => {
            state.search_progress.set_vector_index_ready(true);
            return;
        }
        Ok(false) => {
            state.search_progress.set_vector_index_ready(false);
        }
        Err(err) => {
            tracing::error!(error = %err, "search vector index check failed");
            return;
        }
    }

    *last_attempt = Some(Instant::now());
    tracing::info!(
        ready_sources = counts.ready,
        pending_sources = counts.pending,
        indexing_sources = counts.indexing,
        backlog_threshold = SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD,
        "search vector index build starting"
    );
    let build_span = logfire::span!(
        "worker.search_index.build",
        ready_sources = counts.ready,
        pending_sources = counts.pending,
        indexing_sources = counts.indexing,
        backlog_threshold = SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD,
    );
    match async { db::ensure_vector_index(&conn).await }
        .instrument(build_span)
        .await
    {
        Ok(()) => {
            state.search_progress.set_vector_index_ready(true);
            tracing::info!("search vector index build complete");
        }
        Err(err) => {
            tracing::error!(error = %err, "search vector index build failed");
        }
    }
}

pub fn spawn_search_index_worker(state: AppState) {
    let span = logfire::span!(
        "worker.search_index",
        backfill_scan_limit = SEARCH_BACKFILL_SCAN_LIMIT,
        index_scan_limit = SEARCH_INDEX_SCAN_LIMIT,
        active_poll_interval_secs = SEARCH_INDEX_POLL_INTERVAL.as_secs(),
        idle_poll_start_secs = SEARCH_INDEX_IDLE_POLL_INTERVAL.as_secs(),
        idle_poll_max_secs = SEARCH_INDEX_IDLE_POLL_MAX_INTERVAL.as_secs(),
        vector_index_build_backlog_threshold = SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD,
        reconcile_interval_secs = SEARCH_RECONCILE_INTERVAL.as_secs(),
        vector_index_retry_interval_secs = SEARCH_VECTOR_INDEX_RETRY_INTERVAL.as_secs(),
        auto_create_vector_index = state.search_auto_create_vector_index,
        semantic_enabled = state.search.semantic_enabled(),
        model = state.search.model_label().to_string(),
    );

    tokio::spawn(
        async move {
            tracing::info!(
                backfill_scan_limit = SEARCH_BACKFILL_SCAN_LIMIT,
                index_scan_limit = SEARCH_INDEX_SCAN_LIMIT,
                active_poll_interval_secs = SEARCH_INDEX_POLL_INTERVAL.as_secs(),
                idle_poll_start_secs = SEARCH_INDEX_IDLE_POLL_INTERVAL.as_secs(),
                idle_poll_max_secs = SEARCH_INDEX_IDLE_POLL_MAX_INTERVAL.as_secs(),
                vector_index_build_backlog_threshold = SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD,
                reconcile_interval_secs = SEARCH_RECONCILE_INTERVAL.as_secs(),
                vector_index_retry_interval_secs = SEARCH_VECTOR_INDEX_RETRY_INTERVAL.as_secs(),
                auto_create_vector_index = state.search_auto_create_vector_index,
                semantic_enabled = state.search.semantic_enabled(),
                model = %state.search.model_label(),
                "search index worker started"
            );

            let _ = backfill_search_sources(&state).await;
            let _ = process_pending_search_sources(&state).await;
            let _ = reconcile_search_sources(&state).await;
            let _ = prune_stale_search_rows(&state).await;
            let mut last_vector_index_attempt = None;
            maybe_ensure_vector_index(&state, &mut last_vector_index_attempt).await;
            let mut last_reconcile_at = Instant::now();
            let mut backoff_state = PollBackoffState::default();

            loop {
                let mut had_activity = backfill_search_sources(&state).await;
                had_activity |= process_pending_search_sources(&state).await;
                had_activity |= prune_stale_search_rows(&state).await;
                maybe_ensure_vector_index(&state, &mut last_vector_index_attempt).await;

                if last_reconcile_at.elapsed() >= SEARCH_RECONCILE_INTERVAL {
                    had_activity |= reconcile_search_sources(&state).await;
                    maybe_ensure_vector_index(&state, &mut last_vector_index_attempt).await;
                    last_reconcile_at = Instant::now();
                }

                sleep_with_backoff(SEARCH_INDEX_POLL_BACKOFF, &mut backoff_state, had_activity)
                    .await;
            }
        }
        .instrument(span),
    );
}
