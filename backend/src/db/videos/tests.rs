use chrono::TimeZone;

use crate::models::{CanonicalVideoRecord, ContentStatus, Video};

fn video(id: &str, transcript_status: ContentStatus) -> Video {
    Video {
        id: id.to_string(),
        channel_id: "channel-1".to_string(),
        title: format!("Video {id}"),
        thumbnail_url: None,
        published_at: chrono::Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
        is_short: false,
        transcript_status,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}

fn canonical(id: &str, transcript_status: ContentStatus) -> CanonicalVideoRecord {
    CanonicalVideoRecord {
        id: id.to_string(),
        channel_id: "channel-1".to_string(),
        title: format!("Video {id}"),
        thumbnail_url: None,
        published_at: chrono::Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
        is_short: false,
        transcript_status,
        summary_status: ContentStatus::Pending,
        retry_count: 0,
        quality_score: None,
    }
}

#[tokio::test]
async fn reconcile_sql_videos_with_records_inserts_missing_canonical_rows_and_prunes_stale_rows() {
    let store = crate::db::Store::for_test().await;
    super::super::sql_videos::sql_insert_video(&store, &video("stale", ContentStatus::Pending))
        .await
        .expect("insert stale video");

    let (reconciled, pruned) = super::reconcile_sql_videos_with_records(
        &store,
        vec![canonical("kept", ContentStatus::Ready)],
    )
    .await
    .expect("reconcile videos");

    assert_eq!(reconciled, 1);
    assert_eq!(pruned, 1);
    assert!(
        super::super::sql_videos::sql_get_video(&store, "stale", false)
            .await
            .expect("load stale video")
            .is_none()
    );
    let kept = super::super::sql_videos::sql_get_video(&store, "kept", false)
        .await
        .expect("load kept video")
        .expect("kept video exists");
    assert_eq!(kept.transcript_status, ContentStatus::Ready);
}

#[tokio::test]
async fn reconcile_sql_videos_with_records_overwrites_stale_status_from_canonical_record() {
    let store = crate::db::Store::for_test().await;
    super::super::sql_videos::sql_insert_video(&store, &video("video-1", ContentStatus::Pending))
        .await
        .expect("insert stale status video");

    let (reconciled, pruned) = super::reconcile_sql_videos_with_records(
        &store,
        vec![canonical("video-1", ContentStatus::Ready)],
    )
    .await
    .expect("reconcile videos");

    assert_eq!(reconciled, 1);
    assert_eq!(pruned, 0);
    let video = super::super::sql_videos::sql_get_video(&store, "video-1", false)
        .await
        .expect("load video")
        .expect("video exists");
    assert_eq!(video.transcript_status, ContentStatus::Ready);
}
