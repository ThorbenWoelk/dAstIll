use chrono::{DateTime, Utc};
use reqwest::Client;
use rss::{Channel as RssChannel, Item};
use std::future::Future;
use std::pin::Pin;

use crate::models::{
    ContentItem, ContentItemKind, ContentPart, ContentPartKind, ContentSource, ContentSourceKind,
    ContentStatus, MediaAsset, MediaAssetKind, ProviderIdentity, ProviderKind, SourceBackingKind,
    SubscriptionContainer, SubscriptionContainerKind,
};

use super::build_http_client;
use super::providers::{
    FeedSourceAdapter, ProviderAdapterError, ResolvedSourceDraft, SyncedSourceBatch,
};

#[derive(Clone)]
pub struct PodcastFeedService {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct PodcastEpisodeMaterial {
    pub item: ContentItem,
    pub show_notes: Option<String>,
    pub watch_url: String,
    pub description: Option<String>,
    pub audio_mime_type: Option<String>,
}

impl PodcastFeedService {
    pub fn new() -> Self {
        Self::with_client(build_http_client())
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    fn parse_feed(content: &[u8]) -> Result<RssChannel, ProviderAdapterError> {
        RssChannel::read_from(content).map_err(|error| {
            ProviderAdapterError::Upstream(format!("podcast RSS parse failed: {error}"))
        })
    }

    async fn fetch_feed(&self, url: &str) -> Result<RssChannel, ProviderAdapterError> {
        let url = url.trim();
        if url.is_empty() {
            return Err(ProviderAdapterError::InvalidInput(
                "podcast RSS sources require a feed URL".to_string(),
            ));
        }

        let bytes = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
            .error_for_status()
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
            .bytes()
            .await
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?;

        Self::parse_feed(&bytes)
    }
}

impl Default for PodcastFeedService {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedSourceAdapter for PodcastFeedService {
    fn resolve_feed_source<'a>(
        &'a self,
        input: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedSourceDraft, ProviderAdapterError>> + Send + 'a>>
    {
        Box::pin(async move {
            let feed = self.fetch_feed(input).await?;
            Ok(build_podcast_resolved_source(input, &feed))
        })
    }

    fn sync_feed_source<'a>(
        &'a self,
        source: &'a ContentSource,
    ) -> Pin<Box<dyn Future<Output = Result<SyncedSourceBatch, ProviderAdapterError>> + Send + 'a>>
    {
        Box::pin(async move {
            if source.provider != ProviderKind::PodcastRss
                || source.source_kind != ContentSourceKind::PodcastSeries
            {
                return Err(ProviderAdapterError::UnsupportedSourceKind(
                    source.source_kind,
                ));
            }

            let feed_url = source.subtitle.as_deref().ok_or_else(|| {
                ProviderAdapterError::InvalidInput(
                    "podcast sources require the feed URL in the subtitle field".to_string(),
                )
            })?;
            let feed = self.fetch_feed(feed_url).await?;
            Ok(build_podcast_sync_batch(source, &feed))
        })
    }
}

fn normalize_feed_id(url: &str) -> String {
    let mut result = String::new();
    let mut previous_was_dash = false;
    for ch in url.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            previous_was_dash = false;
            ch.to_ascii_lowercase()
        } else if previous_was_dash {
            continue;
        } else {
            previous_was_dash = true;
            '-'
        };
        result.push(mapped);
    }

    let trimmed = result.trim_matches('-');
    if trimmed.is_empty() {
        "podcast".to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_rss_date(value: Option<&str>) -> Option<DateTime<Utc>> {
    let raw = value?;
    DateTime::parse_from_rfc2822(raw)
        .map(|value| value.with_timezone(&Utc))
        .or_else(|_| DateTime::parse_from_rfc3339(raw).map(|value| value.with_timezone(&Utc)))
        .ok()
}

fn series_thumbnail(feed: &RssChannel) -> Option<String> {
    feed.image().map(|image| image.url().to_string())
}

fn build_podcast_resolved_source(feed_url: &str, feed: &RssChannel) -> ResolvedSourceDraft {
    let source_id = format!("podcast:rss:{}", normalize_feed_id(feed_url));
    let title = feed.title().trim();
    let container = SubscriptionContainer {
        id: format!("podcast:series:{}", normalize_feed_id(feed_url)),
        kind: SubscriptionContainerKind::Series,
        title: title.to_string(),
        provider: ProviderKind::PodcastRss,
        backing_kind: SourceBackingKind::Feed,
        user_editable: false,
        source_ids: vec![source_id.clone()],
    };

    let source = ContentSource {
        id: source_id,
        provider: ProviderKind::PodcastRss,
        source_kind: ContentSourceKind::PodcastSeries,
        container_id: container.id.clone(),
        container_kind: container.kind,
        backing_kind: SourceBackingKind::Feed,
        title: title.to_string(),
        subtitle: Some(feed_url.to_string()),
        handle: None,
        thumbnail_url: series_thumbnail(feed),
        requires_auth: false,
        public_content_available: true,
        entitled_content_available: true,
        external_ids: vec![ProviderIdentity {
            provider: ProviderKind::PodcastRss,
            external_id: feed_url.to_string(),
        }],
    };

    ResolvedSourceDraft { container, source }
}

fn item_guid(item: &Item) -> Option<String> {
    item.guid()
        .map(|guid| guid.value().to_string())
        .or_else(|| item.link().map(ToString::to_string))
        .or_else(|| {
            item.enclosure()
                .map(|enclosure| enclosure.url().to_string())
        })
}

fn item_summary(item: &Item) -> Option<String> {
    item.content()
        .map(ToString::to_string)
        .or_else(|| item.description().map(ToString::to_string))
}

fn build_podcast_sync_batch(source: &ContentSource, feed: &RssChannel) -> SyncedSourceBatch {
    let mut items = Vec::new();
    let mut parts = Vec::new();
    let mut media_assets = Vec::new();

    for item in feed.items() {
        let Some(external_id) = item_guid(item) else {
            continue;
        };

        let compact_id = normalize_feed_id(&external_id);
        let item_id = format!("podcast:episode:{compact_id}");
        let title = item.title().unwrap_or("Untitled episode").to_string();

        items.push(ContentItem {
            id: item_id.clone(),
            source_id: source.id.clone(),
            provider: ProviderKind::PodcastRss,
            item_kind: ContentItemKind::PodcastEpisode,
            title,
            thumbnail_url: source.thumbnail_url.clone(),
            published_at: parse_rss_date(item.pub_date()),
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::PodcastRss,
                external_id,
            }],
        });

        if item_summary(item).is_some() {
            parts.push(ContentPart {
                id: format!("podcast:show-notes:{compact_id}"),
                source_id: source.id.clone(),
                item_id: item_id.clone(),
                provider: ProviderKind::PodcastRss,
                part_kind: ContentPartKind::ShowNotes,
                status: ContentStatus::Ready,
                text_available: true,
            });
        }

        if let Some(enclosure) = item.enclosure() {
            media_assets.push(MediaAsset {
                id: format!("podcast:audio:{compact_id}"),
                source_id: source.id.clone(),
                item_id,
                provider: ProviderKind::PodcastRss,
                asset_kind: MediaAssetKind::SourceAudio,
                title: "Source audio".to_string(),
                url: Some(enclosure.url().to_string()),
                mime_type: Some(enclosure.mime_type().to_string()),
            });
        }
    }

    SyncedSourceBatch {
        items,
        parts,
        media_assets,
    }
}

impl PodcastFeedService {
    pub fn sync_feed_source_materials<'a>(
        &'a self,
        source: &'a ContentSource,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<PodcastEpisodeMaterial>, ProviderAdapterError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if source.provider != ProviderKind::PodcastRss
                || source.source_kind != ContentSourceKind::PodcastSeries
            {
                return Err(ProviderAdapterError::UnsupportedSourceKind(
                    source.source_kind,
                ));
            }

            let feed_url = source.subtitle.as_deref().ok_or_else(|| {
                ProviderAdapterError::InvalidInput(
                    "podcast sources require the feed URL in the subtitle field".to_string(),
                )
            })?;
            let feed = self.fetch_feed(feed_url).await?;
            let materials = feed
                .items()
                .iter()
                .filter_map(|item| {
                    let external_id = item_guid(item)?;
                    let compact_id = normalize_feed_id(&external_id);
                    let item_id = format!("podcast:episode:{compact_id}");
                    let title = item.title().unwrap_or("Untitled episode").to_string();
                    let watch_url = item
                        .link()
                        .map(ToString::to_string)
                        .or_else(|| {
                            item.enclosure()
                                .map(|enclosure| enclosure.url().to_string())
                        })
                        .unwrap_or_else(|| external_id.clone());
                    let show_notes = item_summary(item);
                    Some(PodcastEpisodeMaterial {
                        item: ContentItem {
                            id: item_id,
                            source_id: source.id.clone(),
                            provider: ProviderKind::PodcastRss,
                            item_kind: ContentItemKind::PodcastEpisode,
                            title,
                            thumbnail_url: source.thumbnail_url.clone(),
                            published_at: parse_rss_date(item.pub_date()),
                            external_ids: vec![ProviderIdentity {
                                provider: ProviderKind::PodcastRss,
                                external_id,
                            }],
                        },
                        description: show_notes.clone(),
                        show_notes,
                        watch_url,
                        audio_mime_type: item
                            .enclosure()
                            .map(|enclosure| enclosure.mime_type().to_string()),
                    })
                })
                .collect();

            Ok(materials)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PodcastFeedService, build_podcast_resolved_source, build_podcast_sync_batch};
    use crate::models::{ContentItemKind, ContentSourceKind, MediaAssetKind, ProviderKind};

    fn sample_feed() -> rss::Channel {
        rss::Channel::read_from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
              <channel>
                <title>Example Podcast</title>
                <link>https://example.com/podcast</link>
                <description>Weekly deep dives</description>
                <image>
                  <url>https://example.com/artwork.jpg</url>
                  <title>Example Podcast</title>
                  <link>https://example.com/podcast</link>
                </image>
                <item>
                  <title>Episode 1</title>
                  <guid>episode-1</guid>
                  <pubDate>Tue, 07 Jan 2025 10:00:00 GMT</pubDate>
                  <description>Episode 1 show notes</description>
                  <enclosure url="https://example.com/audio.mp3" length="42" type="audio/mpeg" />
                </item>
              </channel>
            </rss>"#
                .as_bytes(),
        )
        .expect("rss should parse")
    }

    #[test]
    fn resolve_source_builds_podcast_series_contract() {
        let feed = sample_feed();
        let resolved = build_podcast_resolved_source("https://example.com/feed.xml", &feed);

        assert_eq!(resolved.source.provider, ProviderKind::PodcastRss);
        assert_eq!(
            resolved.source.source_kind,
            ContentSourceKind::PodcastSeries
        );
        assert_eq!(resolved.container.source_ids.len(), 1);
        assert_eq!(
            resolved.source.thumbnail_url.as_deref(),
            Some("https://example.com/artwork.jpg")
        );
    }

    #[test]
    fn sync_batch_maps_episode_show_notes_and_audio() {
        let feed = sample_feed();
        let resolved = build_podcast_resolved_source("https://example.com/feed.xml", &feed);
        let batch = build_podcast_sync_batch(&resolved.source, &feed);

        assert_eq!(batch.items.len(), 1);
        assert_eq!(batch.items[0].item_kind, ContentItemKind::PodcastEpisode);
        assert_eq!(batch.parts.len(), 1);
        assert_eq!(
            batch.parts[0].part_kind,
            crate::models::ContentPartKind::ShowNotes
        );
        assert_eq!(batch.media_assets.len(), 1);
        assert_eq!(
            batch.media_assets[0].asset_kind,
            MediaAssetKind::SourceAudio
        );
    }

    #[test]
    fn service_is_constructible() {
        let _service = PodcastFeedService::new();
    }
}
