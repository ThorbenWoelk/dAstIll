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
