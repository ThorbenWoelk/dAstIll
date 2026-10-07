use chrono::Utc;
use std::time::Duration;

use super::queue::next_queue_task;
use super::search_index::should_build_vector_index;
use super::summary_evaluation::should_run_summary_evaluation;
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

mod summary_evaluation_lifecycle {
    use super::super::summary_evaluation::{
        evaluation_failure_kind, previous_summary_scored_better, store_summary_evaluation,
        summary_needs_regeneration,
    };
    use super::video_with_statuses;
    use crate::db::{self, Store, SummaryEvaluationFailureKind};
    use crate::handlers::content::MAX_SUMMARY_AUTO_REGEN_ATTEMPTS;
    use crate::models::{ContentStatus, Summary, SummaryEvaluationJob, SummaryEvaluationResult};
    use crate::services::summary_evaluator::{
        SummaryEvaluation, SummaryEvaluatorError, UnscorableCause,
    };

    const VIDEO_ID: &str = "video";

    fn summary(content: &str, model_used: &str, quality_score: Option<u8>) -> Summary {
        Summary {
            video_id: VIDEO_ID.to_string(),
            content: content.to_string(),
            model_used: Some(model_used.to_string()),
            quality_score,
            quality_note: quality_score.map(|score| format!("scored {score}")),
            quality_model_used: quality_score.map(|_| "eval-model".to_string()),
            summary_tags: Vec::new(),
            summary_tags_evaluated: quality_score.is_some(),
        }
    }

    fn job(content: &str) -> SummaryEvaluationJob {
        SummaryEvaluationJob {
            video_id: VIDEO_ID.to_string(),
            video_title: "Title".to_string(),
            transcript_text: "transcript".to_string(),
            summary_content: content.to_string(),
        }
    }

    fn scored(score: u8) -> SummaryEvaluation {
        SummaryEvaluation {
            result: SummaryEvaluationResult {
                quality_score: Some(score),
                quality_note: Some(format!("scored {score}")),
                quality_model_used: Some("eval-model".to_string()),
                summary_tags: vec!["Tag".to_string()],
            },
            unscorable_cause: None,
        }
    }

    fn unscorable(cause: UnscorableCause) -> SummaryEvaluation {
        SummaryEvaluation {
            result: SummaryEvaluationResult {
                quality_score: None,
                quality_note: Some("**Unscorable**:\n- reason".to_string()),
                quality_model_used: Some("eval-model".to_string()),
                summary_tags: Vec::new(),
            },
            unscorable_cause: Some(cause),
        }
    }

    async fn store_with_summary(stored: &Summary) -> Store {
        let store = Store::for_test().await;
        let video = video_with_statuses(ContentStatus::Ready, ContentStatus::Ready);
        db::insert_video(&store, &video).await.unwrap();
        db::update_video_summary_status(&store, VIDEO_ID, ContentStatus::Ready)
            .await
            .unwrap();
        db::upsert_summary(&store, stored).await.unwrap();
        store
    }

    async fn summary_status(store: &Store) -> ContentStatus {
        db::get_video(store, VIDEO_ID, false)
            .await
            .unwrap()
            .unwrap()
            .summary_status
    }

    #[test]
    fn only_unusable_answers_count_toward_the_unscorable_limit() {
        assert_eq!(
            evaluation_failure_kind(&SummaryEvaluatorError::ParseFailed(
                "failed to decode structured response: missing field `severity`".to_string()
            )),
            Some(SummaryEvaluationFailureKind::UnusableAnswer)
        );
        assert_eq!(
            evaluation_failure_kind(&SummaryEvaluatorError::EvaluationFailed(
                "Ollama generate request failed (500)".to_string()
            )),
            Some(SummaryEvaluationFailureKind::RequestFailed)
        );
        assert_eq!(
            evaluation_failure_kind(&SummaryEvaluatorError::NotAvailable),
            None
        );
    }

    #[test]
    fn previous_summary_wins_only_with_a_higher_score() {
        assert!(previous_summary_scored_better(Some(6), Some(4)));
        assert!(!previous_summary_scored_better(Some(6), Some(6)));
        assert!(!previous_summary_scored_better(Some(6), Some(8)));
        assert!(previous_summary_scored_better(Some(3), None));
        assert!(!previous_summary_scored_better(None, Some(2)));
    }

    #[test]
    fn manual_summaries_never_need_regeneration() {
        assert!(!summary_needs_regeneration(
            &summary("text", "manual", Some(2)),
            None
        ));
        assert!(!summary_needs_regeneration(
            &summary("text", "manual", None),
            Some(UnscorableCause::Summary)
        ));
        assert!(summary_needs_regeneration(
            &summary("text", "glm-5.1:cloud", Some(6)),
            None
        ));
        assert!(!summary_needs_regeneration(
            &summary("text", "glm-5.1:cloud", Some(7)),
            None
        ));
    }

    #[test]
    fn only_summary_caused_unscorable_results_need_regeneration() {
        let unscored = summary("text", "glm-5.1:cloud", None);
        assert!(summary_needs_regeneration(
            &unscored,
            Some(UnscorableCause::Summary)
        ));
        assert!(!summary_needs_regeneration(
            &unscored,
            Some(UnscorableCause::Transcript)
        ));
    }

    #[tokio::test]
    async fn low_score_queues_regeneration_and_keeps_a_copy() {
        let store = store_with_summary(&summary("first", "glm-5.1:cloud", None)).await;

        let outcome = store_summary_evaluation(&store, &job("first"), &scored(5))
            .await
            .unwrap();

        assert!(outcome.stored);
        assert!(outcome.queued_regeneration);
        assert_eq!(summary_status(&store).await, ContentStatus::Pending);
        let copy = db::take_summary_before_regeneration(&store, VIDEO_ID)
            .await
            .unwrap()
            .expect("copy of the low-scored summary");
        assert_eq!(copy.content, "first");
        assert_eq!(copy.quality_score, Some(5));
    }

    #[tokio::test]
    async fn manual_summary_low_score_is_stored_but_not_queued() {
        let store = store_with_summary(&summary("mine", "manual", None)).await;

        let outcome = store_summary_evaluation(&store, &job("mine"), &scored(3))
            .await
            .unwrap();

        assert!(outcome.stored);
        assert!(!outcome.queued_regeneration);
        assert_eq!(summary_status(&store).await, ContentStatus::Ready);
        let stored = db::get_summary(&store, VIDEO_ID).await.unwrap().unwrap();
        assert_eq!(stored.quality_score, Some(3));
    }

    #[tokio::test]
    async fn evaluation_of_replaced_summary_is_discarded() {
        let store = store_with_summary(&summary("newer text", "glm-5.1:cloud", None)).await;

        let outcome = store_summary_evaluation(&store, &job("older text"), &scored(2))
            .await
            .unwrap();

        assert!(!outcome.stored);
        assert!(!outcome.queued_regeneration);
        assert_eq!(summary_status(&store).await, ContentStatus::Ready);
        let stored = db::get_summary(&store, VIDEO_ID).await.unwrap().unwrap();
        assert_eq!(stored.quality_score, None);
    }

    #[tokio::test]
    async fn malformed_summary_queues_regeneration() {
        let store = store_with_summary(&summary("garbled", "glm-5.1:cloud", None)).await;

        let outcome = store_summary_evaluation(
            &store,
            &job("garbled"),
            &unscorable(UnscorableCause::Summary),
        )
        .await
        .unwrap();

        assert!(outcome.queued_regeneration);
        assert_eq!(summary_status(&store).await, ContentStatus::Pending);
    }

    #[tokio::test]
    async fn bad_transcript_does_not_queue_regeneration() {
        let store = store_with_summary(&summary("fine", "glm-5.1:cloud", None)).await;

        let outcome = store_summary_evaluation(
            &store,
            &job("fine"),
            &unscorable(UnscorableCause::Transcript),
        )
        .await
        .unwrap();

        assert!(outcome.stored);
        assert!(!outcome.queued_regeneration);
        assert_eq!(summary_status(&store).await, ContentStatus::Ready);
    }

    #[tokio::test]
    async fn regeneration_stops_after_max_attempts() {
        let store = store_with_summary(&summary("weak", "glm-5.1:cloud", None)).await;
        for _ in 0..MAX_SUMMARY_AUTO_REGEN_ATTEMPTS {
            db::increment_summary_auto_regen_attempts(&store, VIDEO_ID)
                .await
                .unwrap();
        }

        let outcome = store_summary_evaluation(&store, &job("weak"), &scored(4))
            .await
            .unwrap();

        assert!(!outcome.queued_regeneration);
        assert_eq!(summary_status(&store).await, ContentStatus::Ready);
    }

    #[tokio::test]
    async fn lower_scored_regeneration_restores_previous_summary() {
        let store = store_with_summary(&summary("second", "glm-5.1:cloud", None)).await;
        db::save_summary_before_regeneration(&store, &summary("first", "glm-5.1:cloud", Some(6)))
            .await
            .unwrap();
        db::increment_summary_auto_regen_attempts(&store, VIDEO_ID)
            .await
            .unwrap();

        let outcome = store_summary_evaluation(&store, &job("second"), &scored(3))
            .await
            .unwrap();

        let restored = outcome.restored.expect("previous summary restored");
        assert_eq!(restored.content, "first");
        let stored = db::get_summary(&store, VIDEO_ID).await.unwrap().unwrap();
        assert_eq!(stored.content, "first");
        assert_eq!(stored.quality_score, Some(6));
        // One attempt is left, so the restored summary is queued again and kept as the best copy.
        assert!(outcome.queued_regeneration);
        let copy = db::take_summary_before_regeneration(&store, VIDEO_ID)
            .await
            .unwrap()
            .expect("best copy kept for the next attempt");
        assert_eq!(copy.content, "first");
    }

    #[tokio::test]
    async fn higher_scored_regeneration_is_kept() {
        let store = store_with_summary(&summary("second", "glm-5.1:cloud", None)).await;
        db::save_summary_before_regeneration(&store, &summary("first", "glm-5.1:cloud", Some(5)))
            .await
            .unwrap();

        let outcome = store_summary_evaluation(&store, &job("second"), &scored(9))
            .await
            .unwrap();

        assert!(outcome.restored.is_none());
        assert!(!outcome.queued_regeneration);
        let stored = db::get_summary(&store, VIDEO_ID).await.unwrap().unwrap();
        assert_eq!(stored.content, "second");
        assert_eq!(stored.quality_score, Some(9));
        assert!(
            db::take_summary_before_regeneration(&store, VIDEO_ID)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(summary_status(&store).await, ContentStatus::Ready);
    }
}
