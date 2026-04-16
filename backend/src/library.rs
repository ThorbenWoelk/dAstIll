use crate::db::ChannelSnapshotData;
use crate::models::{
    Channel, ContentAvailability, ContentItem, ContentItemKind, ContentPartKind, ContentProvider,
    ContentSource, LibraryBootstrapPayload, LibrarySectionKind, LibrarySectionSummary,
    ProviderMetadataEntry, SourceArchetype, SourceBackingKind, SubscriptionContainerKind, Video,
    WebsiteFolder,
};

const YOUTUBE_SOURCE_PREFIX: &str = "youtube:channel:";
const YOUTUBE_ITEM_PREFIX: &str = "youtube:video:";

pub fn build_youtube_source_id(channel_id: &str) -> String {
    format!("{YOUTUBE_SOURCE_PREFIX}{channel_id}")
}

pub fn build_youtube_item_id(video_id: &str) -> String {
    format!("{YOUTUBE_ITEM_PREFIX}{video_id}")
}

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

fn youtube_provider_metadata(channel: &Channel) -> Vec<ProviderMetadataEntry> {
    channel
        .handle
        .as_ref()
        .map(|handle| {
            vec![ProviderMetadataEntry {
                key: "handle".to_string(),
                value: handle.clone(),
            }]
        })
        .unwrap_or_default()
}

pub fn map_channel_to_content_source(
    channel: &Channel,
    item_count: Option<usize>,
    unread_count: Option<usize>,
) -> ContentSource {
    ContentSource {
        id: build_youtube_source_id(&channel.id),
        provider: ContentProvider::Youtube,
        section: LibrarySectionKind::VideoChannels,
        archetype: SourceArchetype::VideoChannel,
        container_kind: SubscriptionContainerKind::Series,
        backing: SourceBackingKind::FeedBacked,
        container_id: None,
        title: channel.name.clone(),
        subtitle: channel.handle.clone(),
        thumbnail_url: channel.thumbnail_url.clone(),
        added_at: channel.added_at,
        item_count,
        unread_count,
        provider_metadata: youtube_provider_metadata(channel),
    }
}

fn content_availability_for_video(video: &Video) -> ContentAvailability {
    let available_parts = usize::from(matches!(
        video.transcript_status,
        crate::models::ContentStatus::Ready
    )) + usize::from(matches!(
        video.summary_status,
        crate::models::ContentStatus::Ready
    ));

    if available_parts == 0 {
        ContentAvailability::MetadataOnly
    } else {
        ContentAvailability::Full
    }
}

pub fn map_video_to_content_item(video: &Video) -> ContentItem {
    let mut available_parts = Vec::new();
    if matches!(video.transcript_status, crate::models::ContentStatus::Ready) {
        available_parts.push(ContentPartKind::Transcript);
    }
    if matches!(video.summary_status, crate::models::ContentStatus::Ready) {
        available_parts.push(ContentPartKind::GeneratedSummary);
    }

    ContentItem {
        id: build_youtube_item_id(&video.id),
        source_id: build_youtube_source_id(&video.channel_id),
        provider: ContentProvider::Youtube,
        item_kind: ContentItemKind::Video,
        title: video.title.clone(),
        thumbnail_url: video.thumbnail_url.clone(),
        published_at: video.published_at,
        acknowledged: video.acknowledged,
        availability: content_availability_for_video(video),
        available_parts,
        available_media_assets: Vec::new(),
    }
}

pub fn build_library_bootstrap(
    channels: &[Channel],
    snapshot: Option<&ChannelSnapshotData>,
    website_folders: &[WebsiteFolder],
    selected_source_id: Option<String>,
) -> LibraryBootstrapPayload {
    let selected_source_id = selected_source_id.or_else(|| {
        snapshot
            .as_ref()
            .map(|selected| build_youtube_source_id(&selected.channel.id))
    });

    let selected_channel_id = selected_source_id
        .as_deref()
        .and_then(parse_youtube_source_id);
    let selected_snapshot = snapshot.filter(|candidate| {
        selected_channel_id.is_none_or(|channel_id| candidate.channel.id == channel_id)
    });
    let selected_items = selected_snapshot
        .map(|candidate| {
            candidate
                .videos
                .iter()
                .map(map_video_to_content_item)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let selected_channel_source = selected_snapshot.map(|candidate| {
        map_channel_to_content_source(
            &candidate.channel,
            candidate
                .channel_video_count
                .or(Some(candidate.videos.len())),
            Some(
                candidate
                    .videos
                    .iter()
                    .filter(|video| !video.acknowledged)
                    .count(),
            ),
        )
    });

    let sources = channels
        .iter()
        .map(|channel| {
            if selected_snapshot.is_some_and(|candidate| candidate.channel.id == channel.id) {
                selected_channel_source
                    .clone()
                    .unwrap_or_else(|| map_channel_to_content_source(channel, None, None))
            } else {
                map_channel_to_content_source(channel, None, None)
            }
        })
        .collect::<Vec<_>>();

    let video_item_count = selected_snapshot
        .map(|candidate| {
            candidate
                .channel_video_count
                .unwrap_or(candidate.videos.len())
        })
        .unwrap_or(0);

    LibraryBootstrapPayload {
        sections: vec![
            LibrarySectionSummary {
                kind: LibrarySectionKind::VideoChannels,
                title: "Video channels".to_string(),
                source_count: channels.len(),
                item_count: video_item_count,
                container_kinds: vec![SubscriptionContainerKind::Series],
                backing_kinds: vec![SourceBackingKind::FeedBacked],
            },
            LibrarySectionSummary {
                kind: LibrarySectionKind::Podcasts,
                title: "Podcasts".to_string(),
                source_count: 0,
                item_count: 0,
                container_kinds: vec![SubscriptionContainerKind::Series],
                backing_kinds: vec![SourceBackingKind::FeedBacked],
            },
            LibrarySectionSummary {
                kind: LibrarySectionKind::Publications,
                title: "Publications".to_string(),
                source_count: 0,
                item_count: 0,
                container_kinds: vec![
                    SubscriptionContainerKind::Series,
                    SubscriptionContainerKind::SavedSearch,
                    SubscriptionContainerKind::StandaloneTrackedSource,
                ],
                backing_kinds: vec![
                    SourceBackingKind::FeedBacked,
                    SourceBackingKind::QueryBacked,
                    SourceBackingKind::AuthenticatedSession,
                ],
            },
            LibrarySectionSummary {
                kind: LibrarySectionKind::Websites,
                title: "Websites".to_string(),
                source_count: website_folders.len(),
                item_count: 0,
                container_kinds: vec![SubscriptionContainerKind::Folder],
                backing_kinds: vec![SourceBackingKind::ManuallyCurated],
            },
        ],
        selected_source_id: selected_source_id.clone(),
        selected_source: selected_source_id
            .as_ref()
            .and_then(|source_id| {
                sources
                    .iter()
                    .find(|source| &source.id == source_id)
                    .cloned()
            })
            .or(selected_channel_source),
        selected_items,
        sources,
        website_folders: website_folders.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_library_bootstrap, build_youtube_item_id, build_youtube_source_id,
        parse_youtube_item_id, parse_youtube_source_id, resolve_selected_channel_id,
        resolve_selected_video_id,
    };
    use crate::db::ChannelSnapshotData;
    use crate::models::{Channel, ContentStatus, LibrarySectionKind, Video, WebsiteFolder};
    use chrono::{TimeZone, Utc};

    fn channel(id: &str) -> Channel {
        Channel {
            id: id.to_string(),
            handle: Some(format!("@{id}")),
            name: format!("Channel {id}"),
            thumbnail_url: Some(format!("https://img/{id}.jpg")),
            added_at: Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
            earliest_sync_date: None,
            earliest_sync_date_user_set: false,
        }
    }

    fn video(id: &str, channel_id: &str, acknowledged: bool) -> Video {
        Video {
            id: id.to_string(),
            channel_id: channel_id.to_string(),
            title: format!("Video {id}"),
            thumbnail_url: Some(format!("https://img/{id}.jpg")),
            published_at: Utc.with_ymd_and_hms(2026, 4, 2, 0, 0, 0).unwrap(),
            is_short: false,
            transcript_status: ContentStatus::Ready,
            summary_status: ContentStatus::Ready,
            acknowledged,
            retry_count: 0,
            quality_score: None,
        }
    }

    #[test]
    fn youtube_ids_round_trip() {
        let source_id = build_youtube_source_id("abc");
        let item_id = build_youtube_item_id("vid-1");

        assert_eq!(parse_youtube_source_id(&source_id), Some("abc"));
        assert_eq!(parse_youtube_item_id(&item_id), Some("vid-1"));
    }

    #[test]
    fn resolves_generic_selection_aliases_to_legacy_ids() {
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
    fn builds_library_bootstrap_for_youtube_channels_and_website_folders() {
        let selected_channel = channel("abc");
        let channels = vec![selected_channel.clone(), channel("def")];
        let snapshot = ChannelSnapshotData {
            channel: selected_channel,
            derived_earliest_ready_date: None,
            channel_video_count: Some(12),
            has_more: false,
            next_offset: None,
            videos: vec![video("vid-1", "abc", false), video("vid-2", "abc", true)],
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
            &channels,
            Some(&snapshot),
            &folders,
            Some(build_youtube_source_id("abc")),
        );

        assert_eq!(bootstrap.sources.len(), 2);
        assert_eq!(bootstrap.selected_items.len(), 2);
        assert_eq!(
            bootstrap.selected_source_id.as_deref(),
            Some("youtube:channel:abc")
        );
        assert_eq!(
            bootstrap
                .sections
                .iter()
                .find(|section| section.kind == LibrarySectionKind::VideoChannels)
                .map(|section| (section.source_count, section.item_count)),
            Some((2, 12))
        );
        assert_eq!(
            bootstrap
                .sections
                .iter()
                .find(|section| section.kind == LibrarySectionKind::Websites)
                .map(|section| section.source_count),
            Some(1)
        );
        assert_eq!(bootstrap.website_folders.len(), 1);
    }
}
