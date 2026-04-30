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
        LibrarySectionKind::VideoChannels => {
            source.source_kind == ContentSourceKind::YouTubeChannel
        }
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
#[path = "library_tests.rs"]
mod library_tests;
