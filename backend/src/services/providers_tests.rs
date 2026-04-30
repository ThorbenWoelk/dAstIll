use chrono::Utc;

use super::{ManualWebsiteAdapter, ManualWebsiteAdapterContract, youtube_sync_batch};
use crate::models::{
    ContentSource, ContentSourceKind, ContentStatus, ProviderKind, SourceBackingKind,
    SubscriptionContainerKind, Video,
};

fn youtube_source(id: &str) -> ContentSource {
    ContentSource {
        id: id.to_string(),
        provider: ProviderKind::YouTube,
        source_kind: ContentSourceKind::YouTubeChannel,
        container_id: format!("youtube:series:{id}"),
        container_kind: SubscriptionContainerKind::Series,
        backing_kind: SourceBackingKind::Feed,
        title: format!("Channel {id}"),
        subtitle: None,
        handle: Some(format!("@{id}")),
        thumbnail_url: None,
        requires_auth: false,
        public_content_available: true,
        entitled_content_available: true,
        external_ids: vec![],
    }
}

#[test]
fn youtube_sync_batch_maps_items_and_parts_from_videos() {
    let source = youtube_source("channel-1");
    let videos = vec![Video {
        id: "video-1".to_string(),
        channel_id: "channel-1".to_string(),
        title: "Video 1".to_string(),
        thumbnail_url: None,
        published_at: Utc::now(),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }];

    let batch = youtube_sync_batch(&source, &videos).expect("youtube batch should build");

    assert_eq!(batch.items.len(), 1);
    assert_eq!(batch.parts.len(), 2);
    assert!(batch.media_assets.is_empty());
    assert_eq!(batch.items[0].source_id, "channel-1");
    assert_eq!(batch.parts[0].item_id, "video-1");
}

#[test]
fn manual_website_adapter_builds_folder_and_source_contracts() {
    let adapter = ManualWebsiteAdapterContract;
    let folder = adapter
        .build_folder("folder-1", "Research", 0)
        .expect("folder contract should build");
    let source = adapter
        .build_source(
            "source-1",
            "Example",
            "https://example.com",
            Some("folder-1"),
        )
        .expect("source contract should build");

    assert_eq!(folder.container.kind, SubscriptionContainerKind::Folder);
    assert_eq!(source.source.container_id, "folder-1");
    assert_eq!(source.source.backing_kind, SourceBackingKind::Manual);
    assert_eq!(source.source.source_kind, ContentSourceKind::Website);
}
