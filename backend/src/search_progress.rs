use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use tokio::sync::{RwLock, watch};

use crate::db::{SearchMaterial, SearchProgressMaterial};
use crate::models::SearchStatusPayload;
use crate::services::search::{
    SEARCH_SUMMARY_TARGET_WORDS, SEARCH_TRANSCRIPT_OVERLAP_WORDS, SEARCH_TRANSCRIPT_TARGET_WORDS,
    SearchSourceKind, chunk_summary_content, chunk_transcript_content,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchProgressSourceStatus {
    Pending,
    Indexing,
    Ready,
    Failed,
}

impl SearchProgressSourceStatus {
    fn from_db_value(value: Option<&str>) -> Self {
        match value {
            Some("indexing") => Self::Indexing,
            Some("ready") => Self::Ready,
            Some("failed") => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SearchSourceKey {
    video_id: String,
    source_kind: SearchSourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchSourceProgress {
    status: SearchProgressSourceStatus,
    total_chunk_count: usize,
    embedded_chunk_count: usize,
}

#[derive(Clone)]
pub struct SearchProgress {
    model: String,
    dimensions: usize,
    semantic_enabled: bool,
    available: Arc<AtomicBool>,
    pending: Arc<AtomicUsize>,
    indexing: Arc<AtomicUsize>,
    ready: Arc<AtomicUsize>,
    failed: Arc<AtomicUsize>,
    total_sources: Arc<AtomicUsize>,
    total_chunk_count: Arc<AtomicUsize>,
    embedded_chunk_count: Arc<AtomicUsize>,
    vector_index_ready: Arc<AtomicBool>,
    sources: Arc<RwLock<HashMap<SearchSourceKey, SearchSourceProgress>>>,
    sender: watch::Sender<SearchStatusPayload>,
}

impl SearchProgress {
    pub fn new(model: Option<&str>, dimensions: usize, semantic_enabled: bool) -> Self {
        let payload = SearchStatusPayload {
            available: false,
            model: model.unwrap_or_default().to_string(),
            dimensions: if semantic_enabled { dimensions } else { 0 },
            pending: 0,
            indexing: 0,
            ready: 0,
            failed: 0,
            total_sources: 0,
            total_chunk_count: 0,
            embedded_chunk_count: 0,
            vector_index_ready: false,
            retrieval_mode: "fts_only".to_string(),
        };
        let (sender, _) = watch::channel(payload);

        Self {
            model: model.unwrap_or_default().to_string(),
            dimensions,
            semantic_enabled,
            available: Arc::new(AtomicBool::new(false)),
            pending: Arc::new(AtomicUsize::new(0)),
            indexing: Arc::new(AtomicUsize::new(0)),
            ready: Arc::new(AtomicUsize::new(0)),
            failed: Arc::new(AtomicUsize::new(0)),
            total_sources: Arc::new(AtomicUsize::new(0)),
            total_chunk_count: Arc::new(AtomicUsize::new(0)),
            embedded_chunk_count: Arc::new(AtomicUsize::new(0)),
            vector_index_ready: Arc::new(AtomicBool::new(false)),
            sources: Arc::new(RwLock::new(HashMap::new())),
            sender,
        }
    }

    pub fn snapshot(&self) -> SearchStatusPayload {
        let available = self.semantic_enabled && self.available.load(Ordering::Relaxed);
        let vector_index_ready = self.vector_index_ready.load(Ordering::Relaxed);

        SearchStatusPayload {
            available,
            model: self.model.clone(),
            dimensions: if self.semantic_enabled {
                self.dimensions
            } else {
                0
            },
            pending: self.pending.load(Ordering::Relaxed),
            indexing: self.indexing.load(Ordering::Relaxed),
            ready: self.ready.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            total_sources: self.total_sources.load(Ordering::Relaxed),
            total_chunk_count: self.total_chunk_count.load(Ordering::Relaxed),
            embedded_chunk_count: self.embedded_chunk_count.load(Ordering::Relaxed),
            vector_index_ready,
            retrieval_mode: resolve_retrieval_mode(available, vector_index_ready).to_string(),
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<SearchStatusPayload> {
        self.sender.subscribe()
    }

    pub async fn initialize_from_materials(
        &self,
        materials: &[SearchProgressMaterial],
        available: bool,
        vector_index_ready: bool,
    ) {
        let mut next = HashMap::with_capacity(materials.len());
        for material in materials {
            let total_chunk_count = count_chunks(&material.content, material.source_kind);
            next.insert(
                SearchSourceKey {
                    video_id: material.video_id.clone(),
                    source_kind: material.source_kind,
                },
                SearchSourceProgress {
                    status: SearchProgressSourceStatus::from_db_value(
                        material.index_status.as_deref(),
                    ),
                    total_chunk_count,
                    embedded_chunk_count: resolve_embedded_chunk_count(
                        self.semantic_enabled,
                        &self.model,
                        material.embedding_model.as_deref(),
                        material.index_status.as_deref(),
                        total_chunk_count,
                    ),
                },
            );
        }

        {
            let mut sources = self.sources.write().await;
            *sources = next;
            self.store_aggregates(&sources);
        }

        self.available.store(available, Ordering::Relaxed);
        self.vector_index_ready
            .store(vector_index_ready, Ordering::Relaxed);
        self.broadcast_current_snapshot();
    }

    pub async fn upsert_material(
        &self,
        material: &SearchMaterial,
        status: SearchProgressSourceStatus,
        embedded_chunk_count: usize,
    ) {
        let total_chunk_count = count_chunks(&material.content, material.source_kind);
        self.upsert_source(
            &material.video_id,
            material.source_kind,
            status,
            total_chunk_count,
            embedded_chunk_count.min(total_chunk_count),
        )
        .await;
    }

    pub async fn upsert_source(
        &self,
        video_id: &str,
        source_kind: SearchSourceKind,
        status: SearchProgressSourceStatus,
        total_chunk_count: usize,
        embedded_chunk_count: usize,
    ) {
        {
            let mut sources = self.sources.write().await;
            sources.insert(
                SearchSourceKey {
                    video_id: video_id.to_string(),
                    source_kind,
                },
                SearchSourceProgress {
                    status,
                    total_chunk_count,
                    embedded_chunk_count: embedded_chunk_count.min(total_chunk_count),
                },
            );
            self.store_aggregates(&sources);
        }

        self.broadcast_current_snapshot();
    }

    pub async fn set_source_status(
        &self,
        video_id: &str,
        source_kind: SearchSourceKind,
        status: SearchProgressSourceStatus,
    ) {
        let mut updated = false;
        {
            let mut sources = self.sources.write().await;
            if let Some(source) = sources.get_mut(&SearchSourceKey {
                video_id: video_id.to_string(),
                source_kind,
            }) {
                source.status = status;
                if status != SearchProgressSourceStatus::Ready {
                    source.embedded_chunk_count = 0;
                }
                self.store_aggregates(&sources);
                updated = true;
            }
        }

        if updated {
            self.broadcast_current_snapshot();
        }
    }

    pub async fn remove_source(&self, video_id: &str, source_kind: SearchSourceKind) {
        let removed;
        {
            let mut sources = self.sources.write().await;
            removed = sources
                .remove(&SearchSourceKey {
                    video_id: video_id.to_string(),
                    source_kind,
                })
                .is_some();
            if removed {
                self.store_aggregates(&sources);
            }
        }

        if removed {
            self.broadcast_current_snapshot();
        }
    }

    pub fn set_semantic_available(&self, available: bool) {
        self.available.store(available, Ordering::Relaxed);
        self.broadcast_current_snapshot();
    }

    pub fn set_vector_index_ready(&self, ready: bool) {
        self.vector_index_ready.store(ready, Ordering::Relaxed);
        self.broadcast_current_snapshot();
    }

    fn store_aggregates(&self, sources: &HashMap<SearchSourceKey, SearchSourceProgress>) {
        let mut pending = 0usize;
        let mut indexing = 0usize;
        let mut ready = 0usize;
        let mut failed = 0usize;
        let mut total_chunk_count = 0usize;
        let mut embedded_chunk_count = 0usize;

        for source in sources.values() {
            total_chunk_count += source.total_chunk_count;
            embedded_chunk_count += source.embedded_chunk_count;

            match source.status {
                SearchProgressSourceStatus::Pending => pending += 1,
                SearchProgressSourceStatus::Indexing => indexing += 1,
                SearchProgressSourceStatus::Ready => ready += 1,
                SearchProgressSourceStatus::Failed => failed += 1,
            }
        }

        self.pending.store(pending, Ordering::Relaxed);
        self.indexing.store(indexing, Ordering::Relaxed);
        self.ready.store(ready, Ordering::Relaxed);
        self.failed.store(failed, Ordering::Relaxed);
        self.total_sources.store(sources.len(), Ordering::Relaxed);
        self.total_chunk_count
            .store(total_chunk_count, Ordering::Relaxed);
        self.embedded_chunk_count
            .store(embedded_chunk_count, Ordering::Relaxed);
    }

    fn broadcast_current_snapshot(&self) {
        let next = self.snapshot();
        let _ = self.sender.send_if_modified(|current| {
            if *current == next {
                false
            } else {
                *current = next.clone();
                true
            }
        });
    }
}

fn count_chunks(content: &str, source_kind: SearchSourceKind) -> usize {
    match source_kind {
        SearchSourceKind::Transcript => chunk_transcript_content(
            content,
            SEARCH_TRANSCRIPT_TARGET_WORDS,
            SEARCH_TRANSCRIPT_OVERLAP_WORDS,
            None,
        )
        .len(),
        SearchSourceKind::Summary => {
            chunk_summary_content(content, SEARCH_SUMMARY_TARGET_WORDS).len()
        }
    }
}

fn resolve_embedded_chunk_count(
    semantic_enabled: bool,
    configured_model: &str,
    embedding_model: Option<&str>,
    index_status: Option<&str>,
    total_chunk_count: usize,
) -> usize {
    if !semantic_enabled {
        return 0;
    }

    if index_status != Some("ready") {
        return 0;
    }

    if embedding_model != Some(configured_model) {
        return 0;
    }

    total_chunk_count
}

fn resolve_retrieval_mode(available: bool, vector_index_ready: bool) -> &'static str {
    if !available {
        "fts_only"
    } else if vector_index_ready {
        "hybrid_ann"
    } else {
        "hybrid_exact"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SearchProgress, SearchProgressSourceStatus, resolve_embedded_chunk_count,
        resolve_retrieval_mode,
    };
    use crate::db::{SearchMaterial, SearchProgressMaterial};
    use crate::services::search::SearchSourceKind;

    #[tokio::test]
    async fn initialize_counts_missing_sources_as_pending() {
        let progress = SearchProgress::new(Some("qwen3-embedding"), 512, true);
        progress
            .initialize_from_materials(
                &[SearchProgressMaterial {
                    video_id: "video-1".to_string(),
                    source_kind: SearchSourceKind::Transcript,
                    content: "Alpha beta gamma".to_string(),
                    index_status: None,
                    embedding_model: None,
                }],
                true,
                false,
            )
            .await;

        let snapshot = progress.snapshot();
        assert_eq!(snapshot.pending, 1);
        assert_eq!(snapshot.ready, 0);
        assert_eq!(snapshot.total_sources, 1);
        assert_eq!(snapshot.retrieval_mode, "hybrid_exact");
    }

    #[tokio::test]
    async fn upsert_source_tracks_semantic_chunks() {
        let progress = SearchProgress::new(Some("qwen3-embedding"), 512, true);
        let material = SearchMaterial {
            video_id: "video-1".to_string(),
            channel_id: "channel-1".to_string(),
            channel_name: "Channel".to_string(),
            video_title: "Title".to_string(),
            published_at: "2026-01-01T00:00:00Z".to_string(),
            source_kind: SearchSourceKind::Summary,
            content: "# Overview\n\nSemantic search keeps related matches discoverable."
                .to_string(),
            timed_segments: None,
        };

        progress
            .upsert_material(&material, SearchProgressSourceStatus::Ready, 2)
            .await;

        let snapshot = progress.snapshot();
        assert_eq!(snapshot.ready, 1);
        assert_eq!(snapshot.total_chunk_count, snapshot.embedded_chunk_count);
        assert_eq!(snapshot.total_chunk_count, 1);
    }

    #[test]
    fn embedded_chunk_count_requires_matching_ready_embedding_model() {
        assert_eq!(
            resolve_embedded_chunk_count(
                true,
                "qwen3-embedding",
                Some("qwen3-embedding"),
                Some("ready"),
                4,
            ),
            4,
        );
        assert_eq!(
            resolve_embedded_chunk_count(true, "qwen3-embedding", None, Some("ready"), 4),
            0,
        );
        assert_eq!(
            resolve_embedded_chunk_count(
                true,
                "qwen3-embedding",
                Some("other-model"),
                Some("ready"),
                4,
            ),
            0,
        );
    }

    #[test]
    fn retrieval_mode_reflects_vector_index_state() {
        assert_eq!(resolve_retrieval_mode(false, false), "fts_only");
        assert_eq!(resolve_retrieval_mode(true, false), "hybrid_exact");
        assert_eq!(resolve_retrieval_mode(true, true), "hybrid_ann");
    }
}
