use super::*;

#[tokio::test]
async fn independent_sql_caches_exchange_transcript_jobs() {
    // A result crosses the shared object store without touching either SQL cache.
    let remote = Store::for_test().await;
    let mut local = Store::for_test().await;
    local.objects = remote.objects.clone();
    let request = request_youtube_transcript(&remote, "video").await.unwrap();
    let repeated = request_youtube_transcript(&remote, "video").await.unwrap();
    assert_eq!(request.id, repeated.id);
    let jobs = pending_youtube_transcript_requests(&local).await.unwrap();
    assert_eq!(jobs.len(), 1);
    let transcript = crate::models::Transcript {
        video_id: "video".into(),
        raw_text: Some("A substantive transcript.".into()),
        formatted_markdown: None,
        render_mode: crate::models::TranscriptRenderMode::PlainText,
        timed_text: None,
    };
    save_youtube_transcript_result(
        &local,
        &request,
        &YoutubeTranscriptResult::Ready(transcript),
    )
    .await
    .unwrap();
    assert!(matches!(
        youtube_transcript_result(&remote, &request).await.unwrap(),
        Some(YoutubeTranscriptResult::Ready(_))
    ));
    assert!(
        pending_youtube_transcript_requests(&local)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(super::super::sql_video_count(&remote).await.unwrap(), 0);
    assert_eq!(super::super::sql_video_count(&local).await.unwrap(), 0);
}

#[tokio::test]
async fn reset_cancels_requests_and_ignores_late_results() {
    let store = Store::for_test().await;
    let old = request_youtube_transcript(&store, "video").await.unwrap();
    cancel_youtube_transcript_request(&store, "video")
        .await
        .unwrap();
    let new = request_youtube_transcript(&store, "video").await.unwrap();
    assert_ne!(old.id, new.id);
    save_youtube_transcript_result(
        &store,
        &old,
        &YoutubeTranscriptResult::Failed {
            status: 404,
            message: "gone".into(),
        },
    )
    .await
    .unwrap();
    assert!(
        youtube_transcript_result(&store, &new)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn deferred_requests_remain_pending_after_restart_without_retrying_early() {
    let store = Store::for_test().await;
    let request = request_youtube_transcript(&store, "video").await.unwrap();
    save_youtube_transcript_result(
        &store,
        &request,
        &YoutubeTranscriptResult::Deferred {
            retry_at: chrono::Utc::now() + chrono::Duration::minutes(15),
            message: "rate limited".into(),
        },
    )
    .await
    .unwrap();
    let mut restarted = Store::for_test().await;
    restarted.objects = store.objects.clone();
    assert!(
        pending_youtube_transcript_requests(&restarted)
            .await
            .unwrap()
            .is_empty()
    );
    save_youtube_transcript_result(
        &store,
        &request,
        &YoutubeTranscriptResult::Deferred {
            retry_at: chrono::Utc::now() - chrono::Duration::seconds(1),
            message: "retry".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        pending_youtube_transcript_requests(&restarted)
            .await
            .unwrap()
            .len(),
        1
    );
}
