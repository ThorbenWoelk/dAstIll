use super::{
    content_status_from_str, content_status_to_str, reconcile_video_statuses_from_storage,
    sql_get_video, sql_insert_video,
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

#[tokio::test]
async fn insert_refuses_to_reassign_video_across_channels_on_id_conflict() {
    let store = crate::db::Store::for_test().await;
    let original = Video {
        id: "podcast:episode:episode-1".to_string(),
        channel_id: "podcast:rss:feed-a".to_string(),
        title: "Feed A episode".to_string(),
        thumbnail_url: None,
        published_at: chrono::Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    };
    sql_insert_video(&store, &original)
        .await
        .expect("initial insert should succeed");

    let colliding = Video {
        id: original.id.clone(),
        channel_id: "podcast:rss:feed-b".to_string(),
        title: "Feed B episode with colliding guid".to_string(),
        thumbnail_url: None,
        published_at: chrono::Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Pending,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    };
    sql_insert_video(&store, &colliding)
        .await
        .expect("conflict insert should not error");

    let kept = sql_get_video(&store, &original.id, false)
        .await
        .expect("lookup should succeed")
        .expect("video should still exist");
    assert_eq!(kept.channel_id, "podcast:rss:feed-a");
    assert_eq!(kept.title, "Feed A episode");
    assert_eq!(kept.transcript_status, ContentStatus::Ready);
}
