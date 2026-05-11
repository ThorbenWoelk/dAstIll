use super::{
    SearchProgress, SearchProgressSourceStatus, resolve_embedded_chunk_count,
    resolve_retrieval_mode,
};
use crate::db::{SearchMaterial, SearchProgressMaterial};
use crate::search::SearchSourceKind;

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
        content: "# Overview\n\nSemantic search keeps related matches discoverable.".to_string(),
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
