use chrono::Utc;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

use crate::models::{
    Channel, ContentItem, ContentPart, ContentSource, ContentSourceKind, MediaAsset,
    OpenAlexSavedSearchQuery, ProviderKind, SourceBackingKind, SubscriptionContainer,
    SubscriptionContainerKind, Video, youtube_content_item, youtube_content_parts,
    youtube_content_source, youtube_series_container,
};

use super::youtube::{YouTubeError, YouTubeService};

#[derive(Debug, Clone)]
pub struct ResolvedSourceDraft {
    pub container: SubscriptionContainer,
    pub source: ContentSource,
}

#[derive(Debug, Clone, Default)]
pub struct SyncedSourceBatch {
    pub items: Vec<ContentItem>,
    pub parts: Vec<ContentPart>,
    pub media_assets: Vec<MediaAsset>,
}

#[derive(Debug, Clone)]
pub struct ManualWebsiteFolderDraft {
    pub container: SubscriptionContainer,
}

#[derive(Debug, Clone)]
pub struct ManualWebsiteSourceDraft {
    pub source: ContentSource,
}

#[derive(Debug, Error)]
pub enum ProviderAdapterError {
    #[error("invalid provider input: {0}")]
    InvalidInput(String),
    #[error("provider source kind {0:?} is not supported by this adapter")]
    UnsupportedSourceKind(ContentSourceKind),
    #[error("provider request failed: {0}")]
    Upstream(String),
}

impl From<YouTubeError> for ProviderAdapterError {
    fn from(value: YouTubeError) -> Self {
        match value {
            YouTubeError::InvalidInput => {
                Self::InvalidInput("youtube input was invalid".to_string())
            }
            other => Self::Upstream(other.to_string()),
        }
    }
}

pub trait FeedSourceAdapter {
    fn resolve_feed_source<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedSourceDraft, ProviderAdapterError>> + Send + 'a>>;

    fn sync_feed_source<'a>(
        &'a self,
        source: &'a ContentSource,
    ) -> Pin<Box<dyn Future<Output = Result<SyncedSourceBatch, ProviderAdapterError>> + Send + 'a>>;
}

pub trait QuerySourceAdapter {
    fn resolve_query_source<'a>(
        &'a self,
        query: &'a OpenAlexSavedSearchQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedSourceDraft, ProviderAdapterError>> + Send + 'a>>;
}

pub trait ManualWebsiteAdapter {
    fn build_folder(
        &self,
        folder_id: &str,
        title: &str,
        position: usize,
    ) -> Result<ManualWebsiteFolderDraft, ProviderAdapterError>;

    fn build_source(
        &self,
        source_id: &str,
        title: &str,
        url: &str,
        folder_id: Option<&str>,
    ) -> Result<ManualWebsiteSourceDraft, ProviderAdapterError>;
}

fn derive_youtube_handle(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.starts_with('@') {
        Some(trimmed.to_string())
    } else if !trimmed.starts_with("http") && !trimmed.starts_with("UC") && !trimmed.is_empty() {
        Some(format!("@{trimmed}"))
    } else {
        None
    }
}

fn youtube_channel_shell(
    channel_id: String,
    input: &str,
    title: String,
    thumbnail_url: Option<String>,
) -> Channel {
    Channel {
        id: channel_id,
        handle: derive_youtube_handle(input),
        name: title,
        thumbnail_url,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    }
}

fn youtube_sync_batch(
    source: &ContentSource,
    videos: &[Video],
) -> Result<SyncedSourceBatch, ProviderAdapterError> {
    if source.source_kind != ContentSourceKind::YouTubeChannel {
        return Err(ProviderAdapterError::UnsupportedSourceKind(
            source.source_kind,
        ));
    }

    Ok(SyncedSourceBatch {
        items: videos.iter().map(youtube_content_item).collect(),
        parts: videos.iter().flat_map(youtube_content_parts).collect(),
        media_assets: Vec::new(),
    })
}

impl FeedSourceAdapter for YouTubeService {
    fn resolve_feed_source<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedSourceDraft, ProviderAdapterError>> + Send + 'a>>
    {
        Box::pin(async move {
            let (channel_id, title, resolved_thumbnail) = self.resolve_channel(input).await?;
            let thumbnail_url = match self.fetch_channel_thumbnail(&channel_id).await {
                Ok(Some(url)) => Some(url),
                Ok(None) | Err(_) => resolved_thumbnail,
            };
            let channel = youtube_channel_shell(channel_id, input, title, thumbnail_url);

            Ok(ResolvedSourceDraft {
                container: youtube_series_container(&channel),
                source: youtube_content_source(&channel),
            })
        })
    }

    fn sync_feed_source<'a>(
        &'a self,
        source: &'a ContentSource,
    ) -> Pin<Box<dyn Future<Output = Result<SyncedSourceBatch, ProviderAdapterError>> + Send + 'a>>
    {
        Box::pin(async move {
            let videos = self.fetch_videos(&source.id).await?;
            youtube_sync_batch(source, &videos)
        })
    }
}

pub struct ManualWebsiteAdapterContract;

impl ManualWebsiteAdapter for ManualWebsiteAdapterContract {
    fn build_folder(
        &self,
        folder_id: &str,
        title: &str,
        _position: usize,
    ) -> Result<ManualWebsiteFolderDraft, ProviderAdapterError> {
        let folder_id = folder_id.trim();
        let title = title.trim();
        if folder_id.is_empty() || title.is_empty() {
            return Err(ProviderAdapterError::InvalidInput(
                "manual website folders require both id and title".to_string(),
            ));
        }

        Ok(ManualWebsiteFolderDraft {
            container: SubscriptionContainer {
                id: folder_id.to_string(),
                kind: SubscriptionContainerKind::Folder,
                title: title.to_string(),
                provider: ProviderKind::Manual,
                backing_kind: SourceBackingKind::Manual,
                user_editable: true,
                source_ids: Vec::new(),
            },
        })
    }

    fn build_source(
        &self,
        source_id: &str,
        title: &str,
        url: &str,
        folder_id: Option<&str>,
    ) -> Result<ManualWebsiteSourceDraft, ProviderAdapterError> {
        let source_id = source_id.trim();
        let title = title.trim();
        let url = url.trim();
        if source_id.is_empty() || title.is_empty() || url.is_empty() {
            return Err(ProviderAdapterError::InvalidInput(
                "manual website sources require id, title, and url".to_string(),
            ));
        }

        Ok(ManualWebsiteSourceDraft {
            source: ContentSource {
                id: source_id.to_string(),
                provider: ProviderKind::Website,
                source_kind: ContentSourceKind::Website,
                container_id: folder_id.unwrap_or("manual:websites").to_string(),
                container_kind: folder_id
                    .map_or(SubscriptionContainerKind::StandaloneTrackedSource, |_| {
                        SubscriptionContainerKind::Folder
                    }),
                backing_kind: SourceBackingKind::Manual,
                title: title.to_string(),
                subtitle: Some(url.to_string()),
                handle: None,
                thumbnail_url: None,
                requires_auth: false,
                public_content_available: true,
                entitled_content_available: true,
                external_ids: vec![crate::models::ProviderIdentity {
                    provider: ProviderKind::Website,
                    external_id: url.to_string(),
                }],
            },
        })
    }
}

#[cfg(test)]
mod tests {
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
}
