use crate::models::{
    ChannelSnapshotPayload, ContentSource, ContentSourceKind, LibraryBootstrapPayload,
    LibrarySectionKind, LibrarySectionSummary, SourceBackingKind, SubscriptionContainerKind,
    WebsiteFolder,
};

const YOUTUBE_SOURCE_PREFIX: &str = "youtube:channel:";
const YOUTUBE_ITEM_PREFIX: &str = "youtube:video:";

pub fn parse_youtube_source_id(source_id: &str) -> Option<&str> {
    source_id.strip_prefix(YOUTUBE_SOURCE_PREFIX)
}

pub fn parse_youtube_item_id(item_id: &str) -> Option<&str> {
    item_id.strip_prefix(YOUTUBE_ITEM_PREFIX)
}

pub fn resolve_selected_channel_id(
    selected_channel_id: Option<&str>,
    selected_source_id: Option<&str>,
) -> Option<String> {
    selected_channel_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            selected_source_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .and_then(|value| parse_youtube_source_id(value).or(Some(value)))
                .map(ToOwned::to_owned)
        })
}

pub fn resolve_selected_video_id(
    selected_video_id: Option<&str>,
    selected_item_id: Option<&str>,
) -> Option<String> {
    selected_video_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            selected_item_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .and_then(|value| parse_youtube_item_id(value).or(Some(value)))
                .map(ToOwned::to_owned)
        })
}

fn source_matches_section(source: &ContentSource, section: LibrarySectionKind) -> bool {
    match section {
        LibrarySectionKind::VideoChannels => source.source_kind == ContentSourceKind::YouTubeChannel,
        LibrarySectionKind::Podcasts => source.source_kind == ContentSourceKind::PodcastSeries,
        LibrarySectionKind::Publications => matches!(
            source.source_kind,
            ContentSourceKind::PublicationSeries | ContentSourceKind::SavedSearch
        ),
        LibrarySectionKind::Websites => matches!(
            source.source_kind,
            ContentSourceKind::Website | ContentSourceKind::StandaloneTrackedSource
        ),
    }
}

fn section_summary(
    kind: LibrarySectionKind,
    title: &str,
    sources: &[ContentSource],
    item_count: usize,
    container_kinds: Vec<SubscriptionContainerKind>,
    backing_kinds: Vec<SourceBackingKind>,
) -> LibrarySectionSummary {
    LibrarySectionSummary {
        kind,
        title: title.to_string(),
        source_count: sources
            .iter()
            .filter(|source| source_matches_section(source, kind))
            .count(),
        item_count,
        container_kinds,
        backing_kinds,
    }
}

pub fn build_library_bootstrap(
    sources: &[ContentSource],
    snapshot: Option<&ChannelSnapshotPayload>,
    website_folders: &[WebsiteFolder],
    selected_source_id: Option<String>,
) -> LibraryBootstrapPayload {
    let selected_items = snapshot
        .map(|snapshot| snapshot.items.clone())
        .unwrap_or_default();
    let selected_source = selected_source_id
        .as_ref()
        .and_then(|source_id| sources.iter().find(|source| &source.id == source_id))
        .cloned();
    let selected_item_count = snapshot
        .and_then(|snapshot| snapshot.channel_video_count)
        .unwrap_or(selected_items.len());

    LibraryBootstrapPayload {
        sections: vec![
            section_summary(
                LibrarySectionKind::VideoChannels,
                "Video channels",
                sources,
                selected_item_count,
                vec![SubscriptionContainerKind::Series],
                vec![SourceBackingKind::Feed],
            ),
            section_summary(
                LibrarySectionKind::Podcasts,
                "Podcasts",
                sources,
                0,
                vec![SubscriptionContainerKind::Series],
                vec![SourceBackingKind::Feed],
            ),
            section_summary(
                LibrarySectionKind::Publications,
                "Publications",
                sources,
                0,
                vec![
                    SubscriptionContainerKind::Series,
                    SubscriptionContainerKind::SavedSearch,
                    SubscriptionContainerKind::StandaloneTrackedSource,
                ],
                vec![SourceBackingKind::Feed, SourceBackingKind::Query],
            ),
            LibrarySectionSummary {
                kind: LibrarySectionKind::Websites,
                title: "Websites".to_string(),
                source_count: sources
                    .iter()
                    .filter(|source| source_matches_section(source, LibrarySectionKind::Websites))
                    .count()
                    + website_folders.len(),
                item_count: 0,
                container_kinds: vec![
                    SubscriptionContainerKind::Folder,
                    SubscriptionContainerKind::StandaloneTrackedSource,
                ],
                backing_kinds: vec![SourceBackingKind::Manual],
            },
        ],
        sources: sources.to_vec(),
        selected_source_id,
        selected_source,
        selected_items,
        website_folders: website_folders.to_vec(),
    }
}

#[cfg(test)]
mod tests {
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
}
