use chrono::Utc;
use std::time::Duration;

use super::queue::next_queue_task;
use super::search_index::should_build_vector_index;
use super::summary_evaluation::{
    should_queue_summary_auto_regeneration, should_run_summary_evaluation,
};
use super::{PollBackoff, PollBackoffState, QueueTask, populate_fts_index_from_materials};
use crate::db::{SearchMaterial, SearchSourceCounts};
use crate::models::{AiStatus, ContentStatus, Video};
use crate::search::SearchSourceKind;

fn video_with_statuses(transcript_status: ContentStatus, summary_status: ContentStatus) -> Video {
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

#[test]
fn poll_backoff_uses_idle_start_then_doubles_until_max() {
    let backoff = PollBackoff::new(
        Duration::from_secs(3),
        Duration::from_secs(15),
        Duration::from_secs(60),
    );
    let mut state = PollBackoffState::default();

    assert_eq!(
        backoff.next_interval(&mut state, false),
        Duration::from_secs(15)
    );
    assert_eq!(
        backoff.next_interval(&mut state, false),
        Duration::from_secs(30)
    );
    assert_eq!(
        backoff.next_interval(&mut state, false),
        Duration::from_secs(60)
    );
    assert_eq!(
        backoff.next_interval(&mut state, false),
        Duration::from_secs(60)
    );
}

#[test]
fn poll_backoff_resets_to_active_interval_after_activity() {
    let backoff = PollBackoff::new(
        Duration::from_secs(5),
        Duration::from_secs(15),
        Duration::from_secs(60),
    );
    let mut state = PollBackoffState::default();

    assert_eq!(
        backoff.next_interval(&mut state, false),
        Duration::from_secs(15)
    );
    assert_eq!(
        backoff.next_interval(&mut state, false),
        Duration::from_secs(30)
    );
    assert_eq!(
        backoff.next_interval(&mut state, true),
        Duration::from_secs(5)
    );
    assert_eq!(
        backoff.next_interval(&mut state, false),
        Duration::from_secs(15)
    );
}

#[test]
fn vector_index_build_waits_for_backlog_to_shrink_but_not_to_zero() {
    assert!(should_build_vector_index(&SearchSourceCounts {
        pending: 3,
        indexing: 115,
        ready: 6283,
        failed: 0,
        total_sources: 6401,
    }));

    assert!(!should_build_vector_index(&SearchSourceCounts {
        pending: 0,
        indexing: 129,
        ready: 6283,
        failed: 0,
        total_sources: 6412,
    }));

    assert!(!should_build_vector_index(&SearchSourceCounts {
        pending: 0,
        indexing: 0,
        ready: 0,
        failed: 0,
        total_sources: 0,
    }));
}

#[test]
fn parse_bundle_key_preserves_video_ids_with_underscores() {
    let parsed =
        super::parse_bundle_key("search-bundles/video_id_with_underscores_transcript_7.json.gz")
            .expect("bundle key should parse");

    assert_eq!(
        parsed,
        (
            "video_id_with_underscores".to_string(),
            "transcript".to_string(),
            "7".to_string()
        )
    );
}

#[test]
fn parse_chunk_group_key_preserves_video_ids_with_underscores() {
    let parsed = super::parse_chunk_group_key(
        "search-chunks/video_id_with_underscores_summary_hashvalue_12.json",
    )
    .expect("chunk key should parse");

    assert_eq!(
        parsed,
        (
            "video_id_with_underscores".to_string(),
            "summary".to_string()
        )
    );
}

#[tokio::test]
async fn populate_fts_index_hydrates_from_raw_search_material_when_chunks_are_missing() {
    let fts = crate::search::FtsIndex::new()
        .await
        .expect("fts index should be created");
    let materials = vec![
        SearchMaterial {
            video_id: "video-search".to_string(),
            channel_id: "channel-search".to_string(),
            channel_name: "Search Channel".to_string(),
            video_title: "Claude keyword search".to_string(),
            published_at: "2026-04-09T00:00:00Z".to_string(),
            source_kind: SearchSourceKind::Transcript,
            content: "Claude is mentioned in the transcript as a known-good keyword.".to_string(),
            timed_segments: None,
        },
        SearchMaterial {
            video_id: "video-search".to_string(),
            channel_id: "channel-search".to_string(),
            channel_name: "Search Channel".to_string(),
            video_title: "Claude keyword search".to_string(),
            published_at: "2026-04-09T00:00:00Z".to_string(),
            source_kind: SearchSourceKind::Summary,
            content: "Summary also mentions Claude for keyword search verification.".to_string(),
            timed_segments: None,
        },
    ];

    let upserted = populate_fts_index_from_materials(&fts, &materials).await;
    let doc_count = fts.doc_count().await;
    let matches = fts.search("claude", None, None, 10).await;

    assert_eq!(
        upserted, 2,
        "expected transcript and summary sources to hydrate"
    );
    assert!(
        doc_count >= 2,
        "expected transcript and summary chunks to hydrate"
    );
    assert!(
        matches
            .iter()
            .any(|result| result.video_id == "video-search"),
        "expected hydrated FTS index to return the known keyword"
    );
}
