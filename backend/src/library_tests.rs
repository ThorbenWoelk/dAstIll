use super::{
    build_library_bootstrap, parse_youtube_item_id, parse_youtube_source_id,
    resolve_selected_channel_id, resolve_selected_video_id,
};
use crate::models::{
    ChannelSnapshotPayload, ContentItem, ContentItemKind, ContentSource, ContentSourceKind,
    LibrarySectionKind, ProviderKind, SourceBackingKind, SubscriptionContainer,
    SubscriptionContainerKind, SyncDepthPayload, WebsiteFolder,
};
use chrono::{TimeZone, Utc};

#[test]
fn resolves_generic_selection_aliases_to_legacy_ids() {
    assert_eq!(parse_youtube_source_id("youtube:channel:abc"), Some("abc"));
    assert_eq!(parse_youtube_item_id("youtube:video:vid-1"), Some("vid-1"));
    assert_eq!(
        resolve_selected_channel_id(None, Some("youtube:channel:abc")).as_deref(),
        Some("abc")
    );
    assert_eq!(
        resolve_selected_video_id(None, Some("youtube:video:vid-1")).as_deref(),
        Some("vid-1")
    );
}

#[test]
fn builds_library_bootstrap_for_sources_and_website_folders() {
    let source = ContentSource {
        id: "channel-1".to_string(),
        provider: ProviderKind::YouTube,
        source_kind: ContentSourceKind::YouTubeChannel,
        container_id: "youtube:series:channel-1".to_string(),
        container_kind: SubscriptionContainerKind::Series,
        backing_kind: SourceBackingKind::Feed,
        title: "Channel".to_string(),
        subtitle: None,
        handle: None,
        thumbnail_url: None,
        requires_auth: false,
        public_content_available: true,
        entitled_content_available: true,
        external_ids: Vec::new(),
    };
    let item = ContentItem {
        id: "video-1".to_string(),
        source_id: source.id.clone(),
        provider: ProviderKind::YouTube,
        item_kind: ContentItemKind::Video,
        title: "Video".to_string(),
        thumbnail_url: None,
        published_at: None,
        external_ids: Vec::new(),
    };
    let snapshot = ChannelSnapshotPayload {
        channel_id: source.id.clone(),
        source_id: source.id.clone(),
        container: SubscriptionContainer {
            id: source.container_id.clone(),
            kind: source.container_kind,
            title: source.title.clone(),
            provider: source.provider,
            backing_kind: source.backing_kind,
            user_editable: false,
            source_ids: vec![source.id.clone()],
        },
        source: source.clone(),
        sync_depth: SyncDepthPayload {
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
            derived_earliest_ready_date: None,
        },
        channel_video_count: Some(12),
        has_more: false,
        next_offset: None,
        videos: Vec::new(),
        items: vec![item],
        parts: Vec::new(),
    };
    let folders = vec![WebsiteFolder {
        id: "folder-1".to_string(),
        name: "Research".to_string(),
        position: 0,
        created_at: Utc.with_ymd_and_hms(2026, 4, 3, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 4, 3, 0, 0, 0).unwrap(),
        website_count: 0,
    }];

    let bootstrap = build_library_bootstrap(
        &[source],
        Some(&snapshot),
        &folders,
        Some("channel-1".to_string()),
    );

    assert_eq!(bootstrap.selected_items.len(), 1);
    assert_eq!(bootstrap.website_folders.len(), 1);
    assert_eq!(
        bootstrap
            .sections
            .iter()
            .find(|section| section.kind == LibrarySectionKind::VideoChannels)
            .map(|section| (section.source_count, section.item_count)),
        Some((1, 12))
    );
    assert_eq!(
        bootstrap
            .sections
            .iter()
            .find(|section| section.kind == LibrarySectionKind::Websites)
            .map(|section| section.source_count),
        Some(1)
    );
}
