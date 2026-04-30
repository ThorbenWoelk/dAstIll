use super::*;
use tempfile::tempdir;

#[tokio::test]
async fn fts_index_returns_ranked_results() {
    let index = FtsIndex::new().await.expect("index should be created");

    index
        .upsert_source(
            FtsSourceMeta {
                video_id: "video-1",
                source_kind: SearchSourceKind::Transcript,
                channel_id: "channel-a",
                channel_name: "Channel A",
                video_title: "Rust ownership and borrowing",
                published_at: "2026-01-01T00:00:00Z",
            },
            &[
                FtsChunk {
                    chunk_id: "video-1_transcript_1_0".to_string(),
                    section_title: None,
                    chunk_text: "Ownership in Rust prevents dangling pointers at compile time."
                        .to_string(),
                    start_sec: Some(0.0),
                },
                FtsChunk {
                    chunk_id: "video-1_transcript_1_1".to_string(),
                    section_title: None,
                    chunk_text: "Borrowing rules enforce safe concurrent access to data."
                        .to_string(),
                    start_sec: Some(30.0),
                },
            ],
        )
        .await
        .expect("source should be indexed");

    index
        .upsert_source(
            FtsSourceMeta {
                video_id: "video-2",
                source_kind: SearchSourceKind::Summary,
                channel_id: "channel-b",
                channel_name: "Channel B",
                video_title: "Python async patterns",
                published_at: "2026-01-02T00:00:00Z",
            },
            &[FtsChunk {
                chunk_id: "video-2_summary_1_0".to_string(),
                section_title: Some("Overview".to_string()),
                chunk_text: "Async programming in Python using asyncio coroutines.".to_string(),
                start_sec: None,
            }],
        )
        .await
        .expect("source should be indexed");

    let results = index.search("ownership rust", None, None, 10).await;
    assert!(
        !results.is_empty(),
        "should return results for 'ownership rust'"
    );
    assert_eq!(results[0].video_id, "video-1");
    assert_eq!(results[0].start_sec, Some(0.0));
}

#[tokio::test]
async fn fts_index_filters_by_channel_id() {
    let index = FtsIndex::new().await.expect("index should be created");

    for (vid, cid) in [("v1", "ch-a"), ("v2", "ch-b")] {
        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: vid,
                    source_kind: SearchSourceKind::Transcript,
                    channel_id: cid,
                    channel_name: cid,
                    video_title: "semantic search",
                    published_at: "2026-01-01T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id: format!("{vid}_transcript_1_0"),
                    section_title: None,
                    chunk_text: "semantic search with vector embeddings".to_string(),
                    start_sec: None,
                }],
            )
            .await
            .expect("source should be indexed");
    }

    let results = index
        .search("semantic search", None, Some("ch-a"), 10)
        .await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].video_id, "v1");
}

#[tokio::test]
async fn fts_index_preserves_exact_phrase_hits_for_common_title_searches() {
    let index = FtsIndex::new().await.expect("index should be created");

    index
        .upsert_source(
            FtsSourceMeta {
                video_id: "target",
                source_kind: SearchSourceKind::Summary,
                channel_id: "channel-a",
                channel_name: "Channel A",
                video_title: "Anthropic shock wave + One Good Thing",
                published_at: "2026-01-10T00:00:00Z",
            },
            &[FtsChunk {
                chunk_id: "target_summary_1_0".to_string(),
                section_title: Some("One Good Thing".to_string()),
                chunk_text: "One Good Thing closes the episode after the Sam Altman interview."
                    .to_string(),
                start_sec: None,
            }],
        )
        .await
        .expect("target source should be indexed");

    for index_id in 0..6 {
        let video_id = format!("distractor-{index_id}");
        let video_title = format!("Distractor {index_id}");
        let chunk_id = format!("distractor-{index_id}_transcript_1_0");
        let chunk_text =
            format!("A good thing about this demo {index_id} is that the thing works.");
        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: &video_id,
                    source_kind: SearchSourceKind::Transcript,
                    channel_id: "channel-b",
                    channel_name: "Channel B",
                    video_title: &video_title,
                    published_at: "2026-01-01T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id,
                    section_title: None,
                    chunk_text,
                    start_sec: None,
                }],
            )
            .await
            .expect("distractor source should be indexed");
    }

    let results = index.search("one good thing", None, None, 5).await;
    assert!(
        !results.is_empty(),
        "exact phrase search should return results"
    );
    assert_eq!(results[0].video_id, "target");
}

#[tokio::test]
async fn fts_index_handles_natural_language_video_queries() {
    let index = FtsIndex::new().await.expect("index should be created");

    index
        .upsert_source(
            FtsSourceMeta {
                video_id: "target",
                source_kind: SearchSourceKind::Summary,
                channel_id: "channel-a",
                channel_name: "Channel A",
                video_title:
                    "A.I. Backlash Turns Violent + Kara Swisher on Healthmaxxing + The Zuck Bot Is Coming",
                published_at: "2026-01-10T00:00:00Z",
            },
            &[FtsChunk {
                chunk_id: "target_summary_1_0".to_string(),
                section_title: Some("AI Backlash".to_string()),
                chunk_text: "This episode covers the public AI backlash and an attempted attack on Sam Altman."
                    .to_string(),
                start_sec: None,
            }],
        )
        .await
        .expect("target source should be indexed");

    for (index_id, title, chunk_text) in [
        (
            0,
            "I was wrong about GPT-5",
            "This video debates new AI benchmarks and model releases.",
        ),
        (
            1,
            "Can AI Games Be Good?",
            "A discussion about whether AI-generated games can be fun to play.",
        ),
        (
            2,
            "We need to talk about the Claude Code rate limits",
            "A breakdown of the current Claude Code rate limit problems.",
        ),
    ] {
        let video_id = format!("distractor-{index_id}");
        let chunk_id = format!("distractor-{index_id}_summary_1_0");
        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: &video_id,
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-b",
                    channel_name: "Channel B",
                    video_title: title,
                    published_at: "2026-01-01T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id,
                    section_title: None,
                    chunk_text: chunk_text.to_string(),
                    start_sec: None,
                }],
            )
            .await
            .expect("distractor source should be indexed");
    }

    let results = index
        .search("find videos about AI backlash", None, None, 5)
        .await;
    assert!(
        !results.is_empty(),
        "natural language search should return results"
    );
    assert_eq!(results[0].video_id, "target");
}

#[tokio::test]
async fn fts_index_handles_conversational_phrase_queries() {
    let index = FtsIndex::new().await.expect("index should be created");

    index
        .upsert_source(
            FtsSourceMeta {
                video_id: "target",
                source_kind: SearchSourceKind::Summary,
                channel_id: "channel-a",
                channel_name: "Channel A",
                video_title:
                    "Anthropic’s Cybersecurity Shock Wave + Ronan Farrow and Andrew Marantz on Their Sam Altman Investigation + One Good Thing",
                published_at: "2026-01-10T00:00:00Z",
            },
            &[FtsChunk {
                chunk_id: "target_summary_1_0".to_string(),
                section_title: Some("One Good Thing".to_string()),
                chunk_text: "One Good Thing closes the episode after the Sam Altman interview."
                    .to_string(),
                start_sec: None,
            }],
        )
        .await
        .expect("target source should be indexed");

    for (index_id, title, chunk_text) in [
        (
            0,
            "The talk that changed the web",
            "A broad discussion about web standards and browser politics.",
        ),
        (
            1,
            "We need to talk about founder mode",
            "A creator commentary video about management culture and startups.",
        ),
    ] {
        let video_id = format!("distractor-{index_id}");
        let chunk_id = format!("distractor-{index_id}_summary_1_0");
        index
            .upsert_source(
                FtsSourceMeta {
                    video_id: &video_id,
                    source_kind: SearchSourceKind::Summary,
                    channel_id: "channel-b",
                    channel_name: "Channel B",
                    video_title: title,
                    published_at: "2026-01-01T00:00:00Z",
                },
                &[FtsChunk {
                    chunk_id,
                    section_title: None,
                    chunk_text: chunk_text.to_string(),
                    start_sec: None,
                }],
            )
            .await
            .expect("distractor source should be indexed");
    }

    let results = index
        .search("video where they talk about one good thing", None, None, 5)
        .await;
    assert!(
        !results.is_empty(),
        "conversational phrase search should return results"
    );
    assert_eq!(results[0].video_id, "target");
}

#[tokio::test]
async fn fts_index_delete_source_removes_documents() {
    let index = FtsIndex::new().await.expect("index should be created");

    index
        .upsert_source(
            FtsSourceMeta {
                video_id: "video-del",
                source_kind: SearchSourceKind::Transcript,
                channel_id: "ch",
                channel_name: "Ch",
                video_title: "deletion test",
                published_at: "2026-01-01T00:00:00Z",
            },
            &[FtsChunk {
                chunk_id: "video-del_transcript_1_0".to_string(),
                section_title: None,
                chunk_text: "this document should be deleted later".to_string(),
                start_sec: None,
            }],
        )
        .await
        .expect("source should be indexed");

    let before = index.search("deleted", None, None, 10).await;
    assert!(!before.is_empty());

    index
        .delete_source("video-del", SearchSourceKind::Transcript)
        .await
        .expect("delete should succeed");

    let after = index.search("deleted", None, None, 10).await;
    assert!(
        after.is_empty(),
        "deleted source should not appear in results"
    );
}

#[tokio::test]
async fn fts_index_uses_persistent_local_database_file() {
    let temp_dir = tempdir().expect("tempdir should be created");
    let index = FtsIndex::new_in_dir(temp_dir.path())
        .await
        .expect("index should be created");

    index
        .upsert_source(
            FtsSourceMeta {
                video_id: "video-persisted",
                source_kind: SearchSourceKind::Summary,
                channel_id: "channel-persisted",
                channel_name: "Persisted Channel",
                video_title: "Persistent FTS",
                published_at: "2026-01-03T00:00:00Z",
            },
            &[FtsChunk {
                chunk_id: "video-persisted_summary_1_0".to_string(),
                section_title: Some("Restore".to_string()),
                chunk_text: "Persisted libsql search rows should survive reopen.".to_string(),
                start_sec: None,
            }],
        )
        .await
        .expect("source should be indexed");

    let reopened = FtsIndex::new_in_dir(temp_dir.path())
        .await
        .expect("index should reopen");
    let results = reopened.search("persisted reopen", None, None, 10).await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].video_id, "video-persisted");
}
