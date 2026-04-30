use std::time::Duration;

use tokio::time::sleep;

use crate::state::AppState;

const QUEUE_SCAN_LIMIT: usize = 4;
const QUEUE_POLL_INTERVAL: Duration = Duration::from_secs(5);
const QUEUE_IDLE_POLL_INTERVAL: Duration = Duration::from_secs(15);
const QUEUE_IDLE_POLL_MAX_INTERVAL: Duration = Duration::from_secs(60);
const CHANNEL_REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);
const CHANNEL_GAP_SCAN_INTERVAL: Duration = Duration::from_secs(10 * 60);
const CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL: usize = 8;
const SUMMARY_EVAL_SCAN_LIMIT: usize = 4;
const SUMMARY_EVAL_POLL_INTERVAL: Duration = Duration::from_secs(7);
const SUMMARY_EVAL_IDLE_POLL_INTERVAL: Duration = Duration::from_secs(30);
const SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL: Duration = Duration::from_secs(120);
const SEARCH_BACKFILL_SCAN_LIMIT: usize = 64;
const SEARCH_INDEX_SCAN_LIMIT: usize = 8;
const SEARCH_RECONCILE_SCAN_LIMIT: usize = 64;
const SEARCH_PRUNE_SCAN_LIMIT: usize = 256;
const SEARCH_INDEX_POLL_INTERVAL: Duration = Duration::from_secs(3);
const SEARCH_INDEX_IDLE_POLL_INTERVAL: Duration = Duration::from_secs(15);
const SEARCH_INDEX_IDLE_POLL_MAX_INTERVAL: Duration = Duration::from_secs(120);
const SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD: usize = 128;
const SEARCH_RECONCILE_INTERVAL: Duration = Duration::from_secs(60);
const SEARCH_VECTOR_INDEX_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const MAX_DISTILLATION_RETRIES: u8 = 3;

mod gap_scan;
mod queue;
mod refresh;
mod search_index;
mod summary_evaluation;

pub use gap_scan::spawn_gap_scan_worker;
pub use queue::spawn_queue_worker;
pub use refresh::spawn_refresh_worker;
pub use search_index::spawn_search_index_worker;
pub use summary_evaluation::spawn_summary_evaluation_worker;

fn parse_bundle_key(key: &str) -> Option<(String, String, String)> {
    let filename = key
        .strip_prefix("search-bundles/")
        .and_then(|value| value.strip_suffix(".json.gz"))?;
    let mut parts = filename.rsplitn(3, '_');
    let generation = parts.next()?;
    let source_kind = parts.next()?;
    let video_id = parts.next()?;
    Some((
        video_id.to_string(),
        source_kind.to_string(),
        generation.to_string(),
    ))
}

fn parse_chunk_group_key(key: &str) -> Option<(String, String)> {
    let filename = key
        .strip_prefix("search-chunks/")
        .and_then(|value| value.strip_suffix(".json"))?;
    let mut parts = filename.rsplitn(4, '_');
    let _chunk_index = parts.next()?;
    let _content_hash = parts.next()?;
    let source_kind = parts.next()?;
    let video_id = parts.next()?;
    Some((video_id.to_string(), source_kind.to_string()))
}

fn fts_chunks_from_material(
    material: &crate::db::SearchMaterial,
) -> Vec<crate::services::fts::FtsChunk> {
    let drafts = match material.source_kind {
        crate::services::search::SearchSourceKind::Transcript => {
            crate::services::search::chunk_transcript_content(
                &material.content,
                crate::services::search::SEARCH_TRANSCRIPT_TARGET_WORDS,
                crate::services::search::SEARCH_TRANSCRIPT_OVERLAP_WORDS,
                material.timed_segments.as_deref(),
            )
        }
        crate::services::search::SearchSourceKind::Summary => {
            crate::services::search::chunk_summary_content(
                &material.content,
                crate::services::search::SEARCH_SUMMARY_TARGET_WORDS,
            )
        }
    };
    let content_hash = crate::services::search::hash_search_content(&material.content);
    drafts
        .into_iter()
        .enumerate()
        .map(|(index, draft)| crate::services::fts::FtsChunk {
            chunk_id: format!(
                "{}_{}_{}_{}",
                material.video_id,
                material.source_kind.as_str(),
                content_hash,
                index
            ),
            section_title: draft.section_title,
            chunk_text: draft.text,
            start_sec: draft.start_sec,
        })
        .collect()
}

async fn populate_fts_index_from_materials(
    fts: &crate::services::FtsIndex,
    materials: &[crate::db::SearchMaterial],
) -> usize {
    let mut upserted = 0usize;

    for material in materials {
        let chunks = fts_chunks_from_material(material);
        if chunks.is_empty() {
            continue;
        }

        if let Err(err) = fts
            .upsert_source(
                crate::services::fts::FtsSourceMeta {
                    video_id: &material.video_id,
                    source_kind: material.source_kind,
                    channel_id: &material.channel_id,
                    channel_name: &material.channel_name,
                    video_title: &material.video_title,
                    published_at: &material.published_at,
                },
                &chunks,
            )
            .await
        {
            tracing::error!(
                video_id = %material.video_id,
                source_kind = material.source_kind.as_str(),
                error = %err,
                "FTS hydration failed to upsert raw material"
            );
            continue;
        }

        upserted += 1;
    }

    upserted
}

async fn load_all_search_materials(
    store: &crate::db::Store,
) -> Result<Vec<crate::db::SearchMaterial>, crate::db::StoreError> {
    let videos = crate::db::load_all_videos(store).await?;
    let mut materials = Vec::new();

    for video in videos {
        if video.summary_status == crate::models::ContentStatus::Ready
            && let Some(material) = crate::db::load_search_material(
                store,
                &video.id,
                crate::services::search::SearchSourceKind::Summary,
            )
            .await?
        {
            materials.push(material);
        }

        if video.transcript_status == crate::models::ContentStatus::Ready
            && let Some(material) = crate::db::load_search_material(
                store,
                &video.id,
                crate::services::search::SearchSourceKind::Transcript,
            )
            .await?
        {
            materials.push(material);
        }
    }

    Ok(materials)
}

async fn fallback_fts_hydration_to_raw_materials(
    state: &AppState,
    store: &crate::db::Store,
) -> bool {
    let materials = match load_all_search_materials(store).await {
        Ok(materials) => materials,
        Err(err) => {
            tracing::error!(error = %err, "FTS hydration: failed to load raw search materials");
            return false;
        }
    };

    let upserted = populate_fts_index_from_materials(state.fts.as_ref(), &materials).await;
    let doc_count = state.fts.doc_count().await;
    tracing::info!(
        sources = upserted,
        doc_count,
        "FTS hydration (raw materials) complete"
    );
    true
}

/// Populate the keyword search index from all ready search chunks stored in S3.
/// Called once at startup when the runtime index is empty so keyword search
/// does not depend on the background worker replaying each source one by one.
pub async fn populate_fts_index_from_store(state: AppState) {
    use crate::services::fts::{FtsChunk, FtsSourceMeta};
    use crate::services::search::SearchSourceKind;

    #[derive(serde::Deserialize)]
    struct ChunkData {
        section_title: Option<String>,
        chunk_text: String,
        #[serde(default)]
        start_sec: Option<f32>,
    }

    let store = state.db.connect();

    // 1. Try to load from bundles first (Optimized path)
    let bundle_keys = match store.list_keys("search-bundles/").await {
        Ok(keys) => keys,
        Err(err) => {
            tracing::error!(error = %err, "FTS hydration: failed to list bundle keys");
            Vec::new()
        }
    };

    if !bundle_keys.is_empty() {
        tracing::info!(
            bundles = bundle_keys.len(),
            "FTS hydration: starting bundle-based load"
        );

        // Pre-load all videos (hits the 120s cache) and all channels in two bulk fetches
        // instead of one Firestore GET + one S3 GET per video in the loop.
        let video_map: std::collections::HashMap<String, crate::models::Video> =
            crate::db::load_all_videos(&store)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|v| (v.id.clone(), v))
                .collect();
        let channel_map: std::collections::HashMap<String, crate::models::Channel> = store
            .load_all::<crate::models::Channel>("channels/")
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|c| (c.id.clone(), c))
            .collect();

        let mut upserted = 0usize;

        for key in bundle_keys {
            let Some((video_id, source_kind_str, generation)) = parse_bundle_key(&key) else {
                continue;
            };

            let bundle: Vec<ChunkData> = match store.get_json_gz(&key).await {
                Ok(Some(b)) => b,
                _ => continue,
            };

            let fts_chunks: Vec<FtsChunk> = bundle
                .into_iter()
                .enumerate()
                .map(|(index, chunk)| FtsChunk {
                    chunk_id: format!("{video_id}_{source_kind_str}_{generation}_{index}"),
                    section_title: chunk.section_title,
                    chunk_text: chunk.chunk_text,
                    start_sec: chunk.start_sec,
                })
                .collect();

            let Some(video) = video_map.get(&video_id) else {
                continue;
            };
            let channel_name = channel_map
                .get(&video.channel_id)
                .map(|c| c.name.as_str())
                .unwrap_or("");
            let source_kind = SearchSourceKind::from_db_value(&source_kind_str);
            let published_at = video.published_at.to_rfc3339();

            if let Err(err) = state
                .fts
                .upsert_source(
                    FtsSourceMeta {
                        video_id: &video_id,
                        source_kind,
                        channel_id: &video.channel_id,
                        channel_name,
                        video_title: &video.title,
                        published_at: &published_at,
                    },
                    &fts_chunks,
                )
                .await
            {
                tracing::error!(
                    video_id,
                    source_kind = source_kind_str,
                    error = %err,
                    "FTS hydration failed to upsert bundled source"
                );
                continue;
            }
            upserted += 1;
        }

        let doc_count = state.fts.doc_count().await;
        tracing::info!(
            bundles = upserted,
            doc_count,
            "FTS hydration (bundled) complete"
        );
        if doc_count == 0 {
            tracing::warn!(
                "FTS hydration (bundled) produced no documents, falling back to raw materials"
            );
            let _ = fallback_fts_hydration_to_raw_materials(&state, &store).await;
        }
        return;
    }

    // 2. Legacy fallback to individual chunks
    let chunk_keys = match store.list_keys("search-chunks/").await {
        Ok(keys) => keys,
        Err(err) => {
            tracing::error!(error = %err, "FTS hydration: failed to list chunk keys");
            return;
        }
    };

    if chunk_keys.is_empty() {
        let _ = fallback_fts_hydration_to_raw_materials(&state, &store).await;
        return;
    }

    let mut key_groups: std::collections::HashMap<(String, String), Vec<String>> =
        std::collections::HashMap::new();
    for key in chunk_keys {
        let Some((video_id, source_kind)) = parse_chunk_group_key(&key) else {
            continue;
        };
        key_groups
            .entry((video_id, source_kind))
            .or_default()
            .push(key);
    }

    let video_map: std::collections::HashMap<String, crate::models::Video> =
        crate::db::load_all_videos(&store)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|v| (v.id.clone(), v))
            .collect();
    let channel_map: std::collections::HashMap<String, crate::models::Channel> = store
        .load_all::<crate::models::Channel>("channels/")
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    let mut upserted = 0usize;
    for ((video_id, source_kind_str), keys) in key_groups {
        let mut fts_chunks = Vec::with_capacity(keys.len());
        let mut set = tokio::task::JoinSet::new();
        for key in keys {
            let s = store.clone();
            set.spawn(async move {
                s.get_json::<ChunkData>(&key)
                    .await
                    .map(|opt| opt.map(|chunk| (key, chunk)))
            });
        }

        while let Some(result) = set.join_next().await {
            if let Ok(Ok(Some((key, chunk)))) = result {
                let chunk_id = key
                    .strip_prefix("search-chunks/")
                    .and_then(|s| s.strip_suffix(".json"))
                    .unwrap_or(&key)
                    .to_string();
                fts_chunks.push(FtsChunk {
                    chunk_id,
                    section_title: chunk.section_title,
                    chunk_text: chunk.chunk_text,
                    start_sec: chunk.start_sec,
                });
            }
        }

        if fts_chunks.is_empty() {
            continue;
        }

        let Some(video) = video_map.get(&video_id) else {
            continue;
        };
        let channel_name = channel_map
            .get(&video.channel_id)
            .map(|c| c.name.as_str())
            .unwrap_or("");
        let source_kind = SearchSourceKind::from_db_value(&source_kind_str);
        let published_at = video.published_at.to_rfc3339();

        if let Err(err) = state
            .fts
            .upsert_source(
                FtsSourceMeta {
                    video_id: &video_id,
                    source_kind,
                    channel_id: &video.channel_id,
                    channel_name,
                    video_title: &video.title,
                    published_at: &published_at,
                },
                &fts_chunks,
            )
            .await
        {
            tracing::error!(
                video_id,
                source_kind = source_kind_str,
                error = %err,
                "FTS hydration failed to upsert chunk-based source"
            );
            continue;
        }
        upserted += 1;
    }

    let doc_count = state.fts.doc_count().await;
    tracing::info!(
        sources = upserted,
        doc_count,
        "FTS hydration (legacy) complete"
    );
    if doc_count == 0 {
        tracing::warn!(
            "FTS hydration (legacy) produced no documents, falling back to raw materials"
        );
        let _ = fallback_fts_hydration_to_raw_materials(&state, &store).await;
    }
}

#[derive(Clone, Copy, Debug)]
struct PollBackoff {
    active_interval: Duration,
    idle_start_interval: Duration,
    idle_max_interval: Duration,
}

#[derive(Clone, Copy, Debug, Default)]
struct PollBackoffState {
    consecutive_idle_cycles: u32,
}

impl PollBackoff {
    const fn new(
        active_interval: Duration,
        idle_start_interval: Duration,
        idle_max_interval: Duration,
    ) -> Self {
        Self {
            active_interval,
            idle_start_interval,
            idle_max_interval,
        }
    }

    fn next_interval(&self, state: &mut PollBackoffState, had_activity: bool) -> Duration {
        if had_activity {
            state.consecutive_idle_cycles = 0;
            return self.active_interval;
        }

        let multiplier = 1u32
            .checked_shl(state.consecutive_idle_cycles.min(31))
            .unwrap_or(u32::MAX) as u128;
        state.consecutive_idle_cycles = state.consecutive_idle_cycles.saturating_add(1);

        let idle_millis = self.idle_start_interval.as_millis();
        let max_millis = self.idle_max_interval.as_millis();
        let next_millis = idle_millis.saturating_mul(multiplier).min(max_millis);
        let next_millis = next_millis.min(u64::MAX as u128) as u64;
        Duration::from_millis(next_millis)
    }
}

const QUEUE_POLL_BACKOFF: PollBackoff = PollBackoff::new(
    QUEUE_POLL_INTERVAL,
    QUEUE_IDLE_POLL_INTERVAL,
    QUEUE_IDLE_POLL_MAX_INTERVAL,
);
const SUMMARY_EVAL_POLL_BACKOFF: PollBackoff = PollBackoff::new(
    SUMMARY_EVAL_POLL_INTERVAL,
    SUMMARY_EVAL_IDLE_POLL_INTERVAL,
    SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL,
);
const SEARCH_INDEX_POLL_BACKOFF: PollBackoff = PollBackoff::new(
    SEARCH_INDEX_POLL_INTERVAL,
    SEARCH_INDEX_IDLE_POLL_INTERVAL,
    SEARCH_INDEX_IDLE_POLL_MAX_INTERVAL,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QueueTask {
    Transcript,
    Summary,
    Skip,
}

async fn sleep_with_backoff(
    backoff: PollBackoff,
    state: &mut PollBackoffState,
    had_activity: bool,
) {
    let delay = backoff.next_interval(state, had_activity);
    sleep(delay).await;
}

#[cfg(test)]
mod workers_tests;
