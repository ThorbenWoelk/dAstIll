use super::{
    content_status_from_str, content_status_to_str, heal_video_statuses_from_storage,
    reconcile_video_statuses_from_storage,
};
use crate::models::{ContentStatus, Video};

fn build_video() -> Video {
    Video {
        id: "video-1".to_string(),
        channel_id: "channel-1".to_string(),
        title: "Example".to_string(),
        thumbnail_url: None,
        published_at: chrono::Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Pending,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}

#[test]
fn inserted_video_becomes_ready_when_storage_artifacts_exist() {
    let video = build_video();
    let reconciled = reconcile_video_statuses_from_storage(&video, true, true);
    assert_eq!(reconciled.transcript_status, ContentStatus::Ready);
    assert_eq!(reconciled.summary_status, ContentStatus::Ready);
}

#[test]
fn inserted_video_preserves_missing_summary_when_only_transcript_exists() {
    let video = build_video();
    let reconciled = reconcile_video_statuses_from_storage(&video, true, false);
    assert_eq!(reconciled.transcript_status, ContentStatus::Ready);
    assert_eq!(reconciled.summary_status, ContentStatus::Pending);
}

#[test]
fn storage_reconcile_preserves_existing_ready_statuses() {
    let mut video = build_video();
    video.transcript_status = ContentStatus::Ready;
    let reconciled = reconcile_video_statuses_from_storage(&video, false, false);
    assert_eq!(reconciled.transcript_status, ContentStatus::Ready);
    assert_eq!(reconciled.summary_status, ContentStatus::Pending);
}

#[test]
fn content_status_round_trips_through_str() {
    for status in [
        ContentStatus::Pending,
        ContentStatus::Loading,
        ContentStatus::Ready,
        ContentStatus::Failed,
    ] {
        assert_eq!(
            content_status_from_str(content_status_to_str(status)),
            status
        );
    }
}

#[test]
fn heal_keeps_queued_regeneration_pending() {
    let mut video = build_video();
    video.transcript_status = ContentStatus::Ready;
    video.summary_status = ContentStatus::Pending;
    let healed = heal_video_statuses_from_storage(&video, true, true);
    assert_eq!(healed.summary_status, ContentStatus::Pending);
}

#[test]
fn heal_marks_stuck_loading_summary_ready_when_summary_exists() {
    let mut video = build_video();
    video.transcript_status = ContentStatus::Ready;
    video.summary_status = ContentStatus::Loading;
    let healed = heal_video_statuses_from_storage(&video, true, true);
    assert_eq!(healed.summary_status, ContentStatus::Ready);

    video.summary_status = ContentStatus::Failed;
    let healed = heal_video_statuses_from_storage(&video, true, true);
    assert_eq!(healed.summary_status, ContentStatus::Ready);
}

#[test]
fn heal_leaves_loading_summary_without_stored_summary() {
    let mut video = build_video();
    video.transcript_status = ContentStatus::Ready;
    video.summary_status = ContentStatus::Loading;
    let healed = heal_video_statuses_from_storage(&video, true, false);
    assert_eq!(healed.summary_status, ContentStatus::Loading);
}

#[tokio::test]
async fn startup_heal_does_not_cancel_queued_regeneration() {
    let store = crate::db::Store::for_test().await;
    let mut queued = build_video();
    queued.id = "queued".to_string();
    queued.transcript_status = ContentStatus::Ready;
    queued.summary_status = ContentStatus::Pending;
    let mut stuck = build_video();
    stuck.id = "stuck".to_string();
    stuck.transcript_status = ContentStatus::Ready;
    stuck.summary_status = ContentStatus::Loading;

    for video in [&queued, &stuck] {
        super::sql_insert_video(&store, video).await.unwrap();
        super::sql_update_video_transcript_status(&store, &video.id, video.transcript_status)
            .await
            .unwrap();
        super::sql_update_video_summary_status(&store, &video.id, video.summary_status)
            .await
            .unwrap();
        store
            .put_json(
                &format!("summaries/{}.json", video.id),
                &serde_json::json!({ "video_id": video.id, "content": "old summary" }),
            )
            .await
            .unwrap();
    }

    let healed = super::sql_heal_queue_videos(&store, 3).await.unwrap();

    assert_eq!(healed, vec!["stuck".to_string()]);
    let queued_after = super::sql_get_video(&store, "queued", false)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(queued_after.summary_status, ContentStatus::Pending);
    let stuck_after = super::sql_get_video(&store, "stuck", false)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stuck_after.summary_status, ContentStatus::Ready);
}
