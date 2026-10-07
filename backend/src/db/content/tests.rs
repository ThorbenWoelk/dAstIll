use crate::models::Summary;

fn sample_summary(video_id: &str) -> Summary {
    Summary {
        video_id: video_id.to_string(),
        content: "summary".to_string(),
        model_used: Some("summary-model".to_string()),
        quality_score: None,
        quality_note: None,
        quality_model_used: None,
        summary_tags: Vec::new(),
        summary_tags_evaluated: false,
    }
}

#[test]
fn apply_summary_quality_update_marks_tags_as_evaluated_even_when_empty() {
    let mut summary = sample_summary("video-1");

    super::apply_summary_quality_update(
        &mut summary,
        Some(8),
        Some("Solid"),
        Some("eval-model"),
        Some(&Vec::new()),
    );

    assert_eq!(summary.quality_score, Some(8));
    assert_eq!(summary.quality_note.as_deref(), Some("Solid"));
    assert_eq!(summary.quality_model_used.as_deref(), Some("eval-model"));
    assert!(summary.summary_tags.is_empty());
    assert!(summary.summary_tags_evaluated);
}

#[test]
fn summary_needs_quality_eval_skips_completed_empty_tag_evaluations() {
    let mut summary = sample_summary("video-2");

    super::apply_summary_quality_update(
        &mut summary,
        Some(7),
        Some("Good"),
        Some("eval-model"),
        Some(&Vec::new()),
    );

    assert!(!super::summary_needs_quality_eval(&summary));
}

#[test]
fn summary_needs_quality_eval_keeps_legacy_tagless_summaries_pending() {
    let mut summary = sample_summary("video-3");
    summary.quality_score = Some(9);
    summary.quality_note = Some("Legacy evaluation".to_string());
    summary.quality_model_used = Some("old-eval".to_string());

    assert!(super::summary_needs_quality_eval(&summary));
}

mod evaluation_storage {
    use crate::db::{self, Store, SummaryEvaluationFailureKind, SummaryEvaluationFailureOutcome};
    use crate::models::{ContentStatus, Summary, Transcript, TranscriptRenderMode, Video};

    const UNUSABLE: SummaryEvaluationFailureKind = SummaryEvaluationFailureKind::UnusableAnswer;

    fn video(id: &str, published_days_ago: i64) -> Video {
        Video {
            id: id.to_string(),
            channel_id: "channel".to_string(),
            title: format!("Title {id}"),
            thumbnail_url: None,
            published_at: chrono::Utc::now() - chrono::Duration::days(published_days_ago),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }

    fn unscored_summary(video_id: &str, content: &str) -> Summary {
        Summary {
            video_id: video_id.to_string(),
            content: content.to_string(),
            model_used: Some("glm-5.1:cloud".to_string()),
            quality_score: None,
            quality_note: None,
            quality_model_used: None,
            summary_tags: Vec::new(),
            summary_tags_evaluated: false,
        }
    }

    async fn add_video_with_summary(store: &Store, id: &str, published_days_ago: i64) {
        let video = video(id, published_days_ago);
        db::insert_video(store, &video).await.unwrap();
        db::update_video_transcript_status(store, id, ContentStatus::Ready)
            .await
            .unwrap();
        db::update_video_summary_status(store, id, ContentStatus::Ready)
            .await
            .unwrap();
        db::upsert_transcript(
            store,
            &Transcript {
                video_id: id.to_string(),
                raw_text: Some(format!("transcript {id}")),
                formatted_markdown: None,
                render_mode: TranscriptRenderMode::PlainText,
                timed_text: None,
            },
        )
        .await
        .unwrap();
        db::upsert_summary(store, &unscored_summary(id, &format!("summary {id}")))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn update_summary_quality_skips_replaced_summary() {
        let store = Store::for_test().await;
        db::upsert_summary(&store, &unscored_summary("v", "current"))
            .await
            .unwrap();

        let stored =
            db::update_summary_quality(&store, "v", "older", Some(2), Some("bad"), None, None)
                .await
                .unwrap();
        assert!(!stored);
        let summary = db::get_summary(&store, "v").await.unwrap().unwrap();
        assert_eq!(summary.quality_score, None);

        let stored =
            db::update_summary_quality(&store, "v", "current", Some(8), Some("ok"), None, None)
                .await
                .unwrap();
        assert!(stored);
        let summary = db::get_summary(&store, "v").await.unwrap().unwrap();
        assert_eq!(summary.quality_score, Some(8));
    }

    #[tokio::test]
    async fn repeated_evaluation_failures_mark_summary_unscorable() {
        let store = Store::for_test().await;
        add_video_with_summary(&store, "v", 1).await;

        for expected in 1..3 {
            let outcome = db::record_summary_evaluation_failure(
                &store,
                "v",
                "summary v",
                "bad json",
                UNUSABLE,
                3,
            )
            .await
            .unwrap();
            assert_eq!(
                outcome,
                SummaryEvaluationFailureOutcome::WillRetry {
                    failed_attempts: expected
                }
            );
        }
        let outcome = db::record_summary_evaluation_failure(
            &store,
            "v",
            "summary v",
            "bad json",
            UNUSABLE,
            3,
        )
        .await
        .unwrap();
        assert_eq!(
            outcome,
            SummaryEvaluationFailureOutcome::GaveUp {
                unusable_answers: 3
            }
        );

        let summary = db::get_summary(&store, "v").await.unwrap().unwrap();
        assert_eq!(summary.quality_score, None);
        let note = summary.quality_note.expect("unscorable note");
        assert!(note.starts_with("**Unscorable**"));
        assert!(note.contains("bad json"));
        assert!(
            db::list_summaries_pending_quality_eval(&store, 10)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn evaluation_failures_reset_for_a_new_summary() {
        let store = Store::for_test().await;
        add_video_with_summary(&store, "v", 1).await;
        db::record_summary_evaluation_failure(&store, "v", "summary v", "bad json", UNUSABLE, 3)
            .await
            .unwrap();
        assert_eq!(
            db::get_summary_evaluation_failures(&store, "v", "summary v")
                .await
                .unwrap(),
            1
        );

        db::upsert_summary(&store, &unscored_summary("v", "regenerated"))
            .await
            .unwrap();

        assert_eq!(
            db::get_summary_evaluation_failures(&store, "v", "regenerated")
                .await
                .unwrap(),
            0
        );
        let outcome = db::record_summary_evaluation_failure(
            &store,
            "v",
            "summary v",
            "bad json",
            UNUSABLE,
            3,
        )
        .await
        .unwrap();
        assert_eq!(outcome, SummaryEvaluationFailureOutcome::SummaryChanged);
    }

    #[tokio::test]
    async fn evaluation_scan_puts_new_summaries_before_failing_ones() {
        let store = Store::for_test().await;
        // Older videos sort first by id, which used to block the scan forever.
        add_video_with_summary(&store, "a-old-failing", 30).await;
        add_video_with_summary(&store, "b-old", 20).await;
        add_video_with_summary(&store, "c-new", 1).await;
        db::record_summary_evaluation_failure(
            &store,
            "a-old-failing",
            "summary a-old-failing",
            "bad json",
            UNUSABLE,
            3,
        )
        .await
        .unwrap();

        let jobs = db::list_summaries_pending_quality_eval(&store, 2)
            .await
            .unwrap();

        let ids = jobs
            .iter()
            .map(|job| job.video_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["c-new", "b-old"]);
    }

    #[tokio::test]
    async fn manual_summary_clears_regeneration_state() {
        let store = Store::for_test().await;
        add_video_with_summary(&store, "v", 1).await;
        db::save_summary_before_regeneration(&store, &unscored_summary("v", "old"))
            .await
            .unwrap();
        db::record_summary_evaluation_failure(&store, "v", "summary v", "bad json", UNUSABLE, 3)
            .await
            .unwrap();
        db::increment_summary_auto_regen_attempts(&store, "v")
            .await
            .unwrap();

        db::save_manual_summary(&store, "v", "my own words", Some(db::MANUAL_SUMMARY_MODEL))
            .await
            .unwrap();

        assert!(
            db::take_summary_before_regeneration(&store, "v")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            db::get_summary_auto_regen_attempts(&store, "v")
                .await
                .unwrap(),
            0
        );
        let summary = db::get_summary(&store, "v").await.unwrap().unwrap();
        assert!(db::is_manual_summary(&summary));
    }
}

mod evaluation_request_failures {
    use crate::db::{self, Store, SummaryEvaluationFailureKind, SummaryEvaluationFailureOutcome};
    use crate::models::Summary;

    #[tokio::test]
    async fn request_failures_are_retried_without_a_limit() {
        let store = Store::for_test().await;
        db::upsert_summary(
            &store,
            &Summary {
                video_id: "v".to_string(),
                content: "text".to_string(),
                model_used: None,
                quality_score: None,
                quality_note: None,
                quality_model_used: None,
                summary_tags: Vec::new(),
                summary_tags_evaluated: false,
            },
        )
        .await
        .unwrap();

        for expected in 1..=5 {
            let outcome = db::record_summary_evaluation_failure(
                &store,
                "v",
                "text",
                "timeout",
                SummaryEvaluationFailureKind::RequestFailed,
                3,
            )
            .await
            .unwrap();
            assert_eq!(
                outcome,
                SummaryEvaluationFailureOutcome::WillRetry {
                    failed_attempts: expected
                }
            );
        }
        let summary = db::get_summary(&store, "v").await.unwrap().unwrap();
        assert_eq!(summary.quality_note, None);
    }
}
