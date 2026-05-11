use chrono::{DateTime, Utc};
use reqwest::Client;
use rss::extension::Extension;
use rss::{Channel as RssChannel, Item};
use scraper::Html;
use serde_json::Value;
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
use super::transcript::{fetch_public_response, validate_public_media_url};

#[derive(Clone)]
pub struct PodcastFeedService {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct PodcastEpisodeMaterial {
    pub item: ContentItem,
    pub show_notes: Option<String>,
    pub transcript_text: Option<String>,
    pub watch_url: String,
    pub audio_asset: Option<MediaAsset>,
    pub description: Option<String>,
    pub audio_mime_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PodcastTranscriptReference {
    url: String,
    mime_type: String,
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

pub fn podcast_source_id_for_feed_url(feed_url: &str) -> String {
    format!("podcast:rss:{}", normalize_feed_id(feed_url))
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
    let source_id = podcast_source_id_for_feed_url(feed_url);
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

fn podcast_namespace_prefixes(feed: &RssChannel) -> Vec<String> {
    const PODCAST_NAMESPACE: &str = "https://podcastindex.org/namespace/1.0";
    const PODCAST_NAMESPACE_DOC: &str =
        "https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md";

    let mut prefixes = feed
        .namespaces()
        .iter()
        .filter_map(|(prefix, namespace)| {
            (namespace == PODCAST_NAMESPACE || namespace == PODCAST_NAMESPACE_DOC)
                .then(|| prefix.clone())
        })
        .collect::<Vec<_>>();

    if !prefixes.iter().any(|prefix| prefix == "podcast") {
        prefixes.push("podcast".to_string());
    }

    prefixes
}

fn item_transcript_references(feed: &RssChannel, item: &Item) -> Vec<PodcastTranscriptReference> {
    let mut references = Vec::new();
    for prefix in podcast_namespace_prefixes(feed) {
        let Some(extensions) = item
            .extensions()
            .get(&prefix)
            .and_then(|extensions| extensions.get("transcript"))
        else {
            continue;
        };

        references.extend(
            extensions
                .iter()
                .filter_map(transcript_reference_from_extension),
        );
    }

    references.sort_by_key(transcript_reference_rank);
    references
}

fn mime_type_from_url(url: &str) -> Option<&'static str> {
    let path = reqwest::Url::parse(url)
        .ok()
        .map(|url| url.path().to_ascii_lowercase())
        .unwrap_or_else(|| url.to_ascii_lowercase());

    if path.ends_with(".vtt") {
        Some("text/vtt")
    } else if path.ends_with(".srt") {
        Some("application/x-subrip")
    } else if path.ends_with(".json") {
        Some("application/json")
    } else if path.ends_with(".html") || path.ends_with(".htm") {
        Some("text/html")
    } else {
        None
    }
}

fn transcript_reference_from_extension(
    extension: &Extension,
) -> Option<PodcastTranscriptReference> {
    let url = extension.attrs.get("url")?.trim();
    if url.is_empty() {
        return None;
    }

    let mime_type = extension
        .attrs
        .get("type")
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| mime_type_from_url(url).map(ToString::to_string))
        .unwrap_or_else(|| "text/plain".to_string());

    transcript_format(&mime_type, url).map(|_| PodcastTranscriptReference {
        url: url.to_string(),
        mime_type,
    })
}

fn transcript_reference_rank(reference: &PodcastTranscriptReference) -> u8 {
    match transcript_format(&reference.mime_type, &reference.url) {
        Some(TranscriptPayloadFormat::PlainText) => 0,
        Some(TranscriptPayloadFormat::WebVtt) => 1,
        Some(TranscriptPayloadFormat::SubRip) => 2,
        Some(TranscriptPayloadFormat::Json) => 3,
        Some(TranscriptPayloadFormat::Html) => 4,
        None => u8::MAX,
    }
}

fn resolve_transcript_url(
    feed_url: &str,
    transcript_url: &str,
) -> Result<reqwest::Url, ProviderAdapterError> {
    if let Ok(parsed) = reqwest::Url::parse(transcript_url) {
        return Ok(parsed);
    }

    let base = reqwest::Url::parse(feed_url)
        .map_err(|error| ProviderAdapterError::InvalidInput(error.to_string()))?;
    base.join(transcript_url)
        .map_err(|error| ProviderAdapterError::InvalidInput(error.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TranscriptPayloadFormat {
    PlainText,
    WebVtt,
    SubRip,
    Json,
    Html,
}

fn canonical_mime_type(mime_type: &str) -> &str {
    mime_type.split(';').next().unwrap_or("").trim()
}

fn transcript_format(mime_type: &str, url: &str) -> Option<TranscriptPayloadFormat> {
    match canonical_mime_type(mime_type) {
        "text/plain" => Some(TranscriptPayloadFormat::PlainText),
        "text/vtt" => Some(TranscriptPayloadFormat::WebVtt),
        "application/x-subrip" | "application/srt" | "text/srt" => {
            Some(TranscriptPayloadFormat::SubRip)
        }
        "application/json" => Some(TranscriptPayloadFormat::Json),
        "text/html" | "application/xhtml+xml" => Some(TranscriptPayloadFormat::Html),
        _ => match mime_type_from_url(url)? {
            "text/vtt" => Some(TranscriptPayloadFormat::WebVtt),
            "application/x-subrip" => Some(TranscriptPayloadFormat::SubRip),
            "application/json" => Some(TranscriptPayloadFormat::Json),
            "text/html" => Some(TranscriptPayloadFormat::Html),
            _ => None,
        },
    }
}

fn transcript_payload_to_text(body: &str, mime_type: &str, url: &str) -> Option<String> {
    let text = match transcript_format(mime_type, url)? {
        TranscriptPayloadFormat::PlainText => body.to_string(),
        TranscriptPayloadFormat::WebVtt | TranscriptPayloadFormat::SubRip => {
            caption_payload_to_text(body)
        }
        TranscriptPayloadFormat::Json => json_transcript_to_text(body)?,
        TranscriptPayloadFormat::Html => html_transcript_to_text(body),
    };

    normalize_transcript_text(&text)
}

fn normalize_transcript_text(text: &str) -> Option<String> {
    let normalized = text
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    (!normalized.trim().is_empty()).then_some(normalized)
}

fn caption_payload_to_text(body: &str) -> String {
    let mut output = Vec::new();
    let mut block = Vec::new();

    for line in body.lines().chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if let Some(text) = caption_block_to_text(&block) {
                output.push(text);
            }
            block.clear();
        } else {
            block.push(line.trim().to_string());
        }
    }

    output.join("\n")
}

fn caption_block_to_text(block: &[String]) -> Option<String> {
    let first = block.first()?.trim_start_matches('\u{feff}').trim();
    if first.starts_with("WEBVTT")
        || first.starts_with("NOTE")
        || first == "STYLE"
        || first == "REGION"
    {
        return None;
    }

    let timing_index = block.iter().position(|line| line.contains("-->"))?;
    let text = block
        .iter()
        .skip(timing_index + 1)
        .map(|line| strip_inline_markup(line))
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    (!text.is_empty()).then_some(text)
}

fn strip_inline_markup(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;

    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn json_transcript_to_text(body: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    let segments = value.get("segments")?.as_array()?;
    let mut lines = Vec::new();
    let mut current_speaker: Option<String> = None;
    let mut current_words = Vec::new();

    for segment in segments {
        let text = segment
            .get("body")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty());
        let Some(text) = text else {
            continue;
        };

        let speaker = segment
            .get("speaker")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|speaker| !speaker.is_empty())
            .map(ToOwned::to_owned);

        if current_speaker != speaker && !current_words.is_empty() {
            push_json_transcript_line(&mut lines, current_speaker.as_deref(), &current_words);
            current_words.clear();
        }

        current_speaker = speaker;
        current_words.push(text.to_string());
    }

    if !current_words.is_empty() {
        push_json_transcript_line(&mut lines, current_speaker.as_deref(), &current_words);
    }

    Some(lines.join("\n"))
}

fn push_json_transcript_line(lines: &mut Vec<String>, speaker: Option<&str>, words: &[String]) {
    let body = words.join(" ");
    if let Some(speaker) = speaker {
        lines.push(format!("{speaker}: {body}"));
    } else {
        lines.push(body);
    }
}

fn html_transcript_to_text(body: &str) -> String {
    let document = Html::parse_document(body);
    document
        .root_element()
        .text()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
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

        if !item_transcript_references(feed, item).is_empty() {
            parts.push(ContentPart {
                id: format!("podcast:transcript:{compact_id}"),
                source_id: source.id.clone(),
                item_id: item_id.clone(),
                provider: ProviderKind::PodcastRss,
                part_kind: ContentPartKind::Transcript,
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
            let mut materials = Vec::new();
            for item in feed.items() {
                let Some(external_id) = item_guid(item) else {
                    continue;
                };
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
                let audio_asset = item.enclosure().map(|enclosure| MediaAsset {
                    id: format!("podcast:audio:{compact_id}"),
                    source_id: source.id.clone(),
                    item_id: item_id.clone(),
                    provider: ProviderKind::PodcastRss,
                    asset_kind: MediaAssetKind::SourceAudio,
                    title: "Source audio".to_string(),
                    url: Some(enclosure.url().to_string()),
                    mime_type: Some(enclosure.mime_type().to_string()),
                });
                let show_notes = item_summary(item);
                let transcript_text = self.fetch_episode_transcript(feed_url, &feed, item).await;
                materials.push(PodcastEpisodeMaterial {
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
                    transcript_text,
                    watch_url,
                    audio_asset,
                    audio_mime_type: item
                        .enclosure()
                        .map(|enclosure| enclosure.mime_type().to_string()),
                });
            }

            Ok(materials)
        })
    }
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

    async fn fetch_episode_transcript(
        &self,
        feed_url: &str,
        feed: &RssChannel,
        item: &Item,
    ) -> Option<String> {
        for reference in item_transcript_references(feed, item) {
            match self.fetch_transcript_reference(feed_url, &reference).await {
                Ok(Some(transcript)) => return Some(transcript),
                Ok(None) => {}
                Err(err) => {
                    tracing::warn!(
                        transcript_url = %reference.url,
                        error = %err,
                        "podcast transcript fetch failed"
                    );
                }
            }
        }

        None
    }

    async fn fetch_transcript_reference(
        &self,
        feed_url: &str,
        reference: &PodcastTranscriptReference,
    ) -> Result<Option<String>, ProviderAdapterError> {
        let url = resolve_transcript_url(feed_url, &reference.url)?;
        validate_public_media_url(url.as_str())
            .await
            .map_err(|error| ProviderAdapterError::InvalidInput(error.to_string()))?;
        let response = fetch_public_response(url.as_str(), 20)
            .await
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderAdapterError::Upstream(format!(
                "podcast transcript returned HTTP {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?;

        Ok(transcript_payload_to_text(
            &body,
            &reference.mime_type,
            url.as_str(),
        ))
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
