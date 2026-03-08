use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;

use crate::models::{Video, VideoInfo};
use crate::services::build_http_client;
use crate::services::http::YouTubeQuotaCooldown;

#[derive(Error, Debug)]
pub enum YouTubeError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Failed to parse RSS: {0}")]
    RssError(#[from] rss::Error),
    #[error("Failed to parse feed: {0}")]
    FeedParseError(String),
    #[error("Channel not found")]
    ChannelNotFound,
    #[error("Invalid input format")]
    InvalidInput,
}

pub struct YouTubeService {
    client: Client,
    api_key: Option<String>,
    quota_cooldown: Option<Arc<YouTubeQuotaCooldown>>,
}

struct WatchMetadata {
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Default)]
struct WatchVideoDetails {
    title: Option<String>,
    description: Option<String>,
    thumbnail_url: Option<String>,
    channel_name: Option<String>,
    channel_id: Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    duration_iso8601: Option<String>,
    duration_seconds: Option<u64>,
    view_count: Option<u64>,
}

#[derive(Deserialize)]
struct DataApiListResponse<T> {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    items: Option<Vec<T>>,
}

#[derive(Deserialize)]
struct DataApiChannelItem {
    #[serde(rename = "contentDetails")]
    content_details: DataApiChannelContentDetails,
}

#[derive(Deserialize)]
struct DataApiChannelContentDetails {
    #[serde(rename = "relatedPlaylists")]
    related_playlists: DataApiRelatedPlaylists,
}

#[derive(Deserialize)]
struct DataApiRelatedPlaylists {
    uploads: String,
}

#[derive(Deserialize)]
struct DataApiPlaylistItem {
    snippet: DataApiPlaylistItemSnippet,
}

#[derive(Deserialize)]
struct DataApiPlaylistItemSnippet {
    title: String,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    thumbnails: Option<DataApiThumbnails>,
    #[serde(rename = "resourceId")]
    resource_id: Option<DataApiResourceId>,
}

#[derive(Deserialize)]
struct DataApiResourceId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Deserialize)]
struct DataApiThumbnails {
    maxres: Option<DataApiThumbnail>,
    standard: Option<DataApiThumbnail>,
    high: Option<DataApiThumbnail>,
    medium: Option<DataApiThumbnail>,
    default: Option<DataApiThumbnail>,
}

#[derive(Deserialize)]
struct DataApiThumbnail {
    url: Option<String>,
}

impl YouTubeService {
    fn desktop_user_agent() -> &'static str {
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
    }

    pub fn new() -> Self {
        Self::with_client(build_http_client())
    }

    pub fn with_client(client: Client) -> Self {
        let api_key = std::env::var("YOUTUBE_API_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        Self {
            client,
            api_key,
            quota_cooldown: None,
        }
    }

    pub fn with_quota_cooldown(mut self, cooldown: Arc<YouTubeQuotaCooldown>) -> Self {
        self.quota_cooldown = Some(cooldown);
        self
    }

    /// Detects if a response body indicates that the YouTube API quota has been exceeded.
    fn is_quota_exceeded(body: &str) -> bool {
        body.contains("quotaExceeded")
    }

    /// Validates whether the configured YouTube Data API key can make requests.
    /// Returns:
    /// - Ok(None): no API key configured
    /// - Ok(Some(true)): key accepted by the API
    /// - Ok(Some(false)): key rejected by the API
    pub async fn validate_data_api_key(&self) -> Result<Option<bool>, YouTubeError> {
        let Some(api_key) = self.api_key.as_deref() else {
            return Ok(None);
        };

        if self
            .quota_cooldown
            .as_ref()
            .is_some_and(|cd| cd.is_active())
        {
            return Ok(Some(false));
        }

        let response = self
            .client
            .get("https://www.googleapis.com/youtube/v3/channels")
            .query(&[
                ("part", "id"),
                ("id", "UC_x5XG1OV2P6uZZ5FSM9Ttw"),
                ("maxResults", "1"),
                ("key", api_key),
            ])
            .send()
            .await?;

        Ok(Some(response.status().is_success()))
    }

    /// Resolve various input formats to a channel ID and name.
    /// Accepts: @handle, UCxxx channel ID, or full YouTube URL.
    pub async fn resolve_channel(
        &self,
        input: &str,
    ) -> Result<(String, String, Option<String>), YouTubeError> {
        let input = input.trim();

        // Direct channel ID (starts with UC and is 24 chars)
        if input.starts_with("UC")
            && input.len() == 24
            && input
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            let (name, thumb) = self.fetch_channel_info(input).await?;
            return Ok((input.to_string(), name, thumb));
        }

        // Handle format: @handle or just handle
        let handle = if input.starts_with('@') {
            input.to_string()
        } else if input.starts_with("https://") || input.starts_with("http://") {
            // URL format - extract handle or channel ID
            return self.resolve_from_url(input).await;
        } else {
            format!("@{input}")
        };

        self.resolve_from_handle(&handle).await
    }

    async fn resolve_from_url(
        &self,
        url: &str,
    ) -> Result<(String, String, Option<String>), YouTubeError> {
        // Extract channel ID or handle from URL
        if url.contains("/channel/") {
            // https://youtube.com/channel/UCxxx
            if let Some(id) = url.split("/channel/").nth(1) {
                let id = id
                    .split('/')
                    .next()
                    .unwrap_or(id)
                    .split('?')
                    .next()
                    .unwrap_or(id);
                if id.starts_with("UC") && id.len() >= 24 {
                    let id = &id[..24];
                    let (name, thumb) = self.fetch_channel_info(id).await?;
                    return Ok((id.to_string(), name, thumb));
                }
            }
        }

        if url.contains("/@") {
            // https://youtube.com/@handle
            if let Some(handle_part) = url.split("/@").nth(1) {
                let handle_part = handle_part
                    .split('/')
                    .next()
                    .unwrap_or(handle_part)
                    .split('?')
                    .next()
                    .unwrap_or(handle_part);
                let handle = format!("@{handle_part}");
                return self.resolve_from_handle(&handle).await;
            }
        }

        // Try fetching the page directly to find channel ID
        self.fetch_channel_id_from_page(url).await
    }

    async fn resolve_from_handle(
        &self,
        handle: &str,
    ) -> Result<(String, String, Option<String>), YouTubeError> {
        let url = format!("https://www.youtube.com/{handle}");
        self.fetch_channel_id_from_page(&url).await
    }

    async fn fetch_channel_id_from_page(
        &self,
        url: &str,
    ) -> Result<(String, String, Option<String>), YouTubeError> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", Self::desktop_user_agent())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(YouTubeError::ChannelNotFound);
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Look for channel ID in meta tags or page content
        let channel_id = self.extract_channel_id(&document)?;
        let name = self
            .extract_channel_name(&document)
            .unwrap_or_else(|| "Unknown Channel".to_string());
        let thumb = self.extract_channel_thumbnail(&document, &html);

        Ok((channel_id, name, thumb))
    }

    fn extract_channel_id(&self, document: &Html) -> Result<String, YouTubeError> {
        // Try meta tag first
        let meta_selector = Selector::parse(r#"meta[itemprop="channelId"]"#).unwrap();
        if let Some(element) = document.select(&meta_selector).next() {
            if let Some(id) = element.value().attr("content") {
                return Ok(id.to_string());
            }
        }

        // Try link canonical with channel ID
        let link_selector = Selector::parse(r#"link[rel="canonical"]"#).unwrap();
        if let Some(element) = document.select(&link_selector).next() {
            if let Some(href) = element.value().attr("href") {
                if href.contains("/channel/") {
                    if let Some(id) = href.split("/channel/").nth(1) {
                        let id = id.split('/').next().unwrap_or(id);
                        if id.starts_with("UC") {
                            return Ok(id.to_string());
                        }
                    }
                }
            }
        }

        // Search in page source for browseId
        let html = document.html();
        if let Some(pos) = html.find("\"browseId\":\"UC") {
            let start = pos + 12; // length of "\"browseId\":\""
            if let Some(end_quote) = html[start..].find('"') {
                let id = &html[start..start + end_quote];
                if id.len() >= 24 {
                    return Ok(id[..24].to_string());
                }
            }
        }

        Err(YouTubeError::ChannelNotFound)
    }

    fn extract_channel_name(&self, document: &Html) -> Option<String> {
        // Try og:title meta tag
        let og_selector = Selector::parse(r#"meta[property="og:title"]"#).unwrap();
        if let Some(element) = document.select(&og_selector).next() {
            if let Some(title) = element.value().attr("content") {
                // Remove " - YouTube" suffix if present
                let name = title.trim_end_matches(" - YouTube").to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }

        // Try title tag
        let title_selector = Selector::parse("title").unwrap();
        if let Some(element) = document.select(&title_selector).next() {
            let title = element.text().collect::<String>();
            let name = title.trim_end_matches(" - YouTube").to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }

        None
    }

    fn extract_channel_thumbnail(&self, document: &Html, html: &str) -> Option<String> {
        let og_selector = Selector::parse(r#"meta[property="og:image"]"#).unwrap();
        if let Some(url) = document
            .select(&og_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(ToOwned::to_owned)
        {
            return Some(url);
        }

        let itemprop_selector = Selector::parse(r#"link[itemprop="thumbnailUrl"]"#).unwrap();
        if let Some(url) = document
            .select(&itemprop_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(ToOwned::to_owned)
        {
            return Some(url);
        }

        let image_src_selector = Selector::parse(r#"link[rel="image_src"]"#).unwrap();
        if let Some(url) = document
            .select(&image_src_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(ToOwned::to_owned)
        {
            return Some(url);
        }

        Self::extract_yt3_thumbnail_from_html(html)
    }

    fn extract_yt3_thumbnail_from_html(html: &str) -> Option<String> {
        let markers = [
            "https://yt3.googleusercontent.com/",
            "https://yt3.ggpht.com/",
        ];

        for marker in markers {
            if let Some(start) = html.find(marker) {
                let end = html[start..]
                    .char_indices()
                    .find_map(|(idx, ch)| {
                        if matches!(ch, '"' | '\'' | '<' | '>') || ch.is_whitespace() {
                            Some(start + idx)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(html.len());

                if end > start {
                    return Some(
                        html[start..end]
                            .replace("\\\\u0026", "&")
                            .replace("\\u0026", "&")
                            .replace("\\/", "/")
                            .replace("&amp;", "&"),
                    );
                }
            }
        }

        None
    }

    async fn fetch_channel_info(
        &self,
        channel_id: &str,
    ) -> Result<(String, Option<String>), YouTubeError> {
        let url = format!("https://www.youtube.com/channel/{channel_id}");
        let (_, name, thumb) = self.fetch_channel_id_from_page(&url).await?;
        Ok((name, thumb))
    }

    pub async fn fetch_channel_thumbnail(
        &self,
        channel_id: &str,
    ) -> Result<Option<String>, YouTubeError> {
        let (_, thumb) = self.fetch_channel_info(channel_id).await?;
        Ok(thumb)
    }

    /// Fetch recent videos from YouTube Data API with RSS fallback.
    pub async fn fetch_videos(&self, channel_id: &str) -> Result<Vec<Video>, YouTubeError> {
        let cooldown_active = self
            .quota_cooldown
            .as_ref()
            .is_some_and(|cd| cd.is_active());

        if let Some(api_key) = self.api_key.as_deref() {
            if cooldown_active {
                tracing::debug!(
                    channel_id = %channel_id,
                    "skipping YouTube Data API due to active quota cooldown, using RSS fallback"
                );
            } else {
                match self
                    .fetch_videos_from_data_api(channel_id, api_key, 25, &HashSet::new(), None)
                    .await
                {
                    Ok((videos, _)) => return Ok(videos),
                    Err(err) => {
                        tracing::warn!(
                            channel_id = %channel_id,
                            error = %err,
                            "YouTube Data API fetch failed, falling back to RSS feed"
                        );
                    }
                }
            }
        }

        let feed_url = format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}");
        let response = self.client.get(&feed_url).send().await?;
        if !response.status().is_success() {
            return Err(YouTubeError::ChannelNotFound);
        }

        let content = response.bytes().await?;
        let mut videos = Self::parse_videos_from_feed(&content, channel_id)?;
        for video in &mut videos {
            video.is_short = self.fetch_is_short_flag(&video.id).await;
        }
        Ok(videos)
    }

    /// Reconcile missing videos from the channel videos list APIs.
    /// Selects only IDs that are not present in `known_video_ids`.
    /// Uses Data API when configured and falls back to InnerTube browse pagination.
    /// Stops if `until` date is reached.
    pub async fn fetch_videos_backfill_missing(
        &self,
        channel_id: &str,
        known_video_ids: &HashSet<String>,
        limit: usize,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<Video>, bool), YouTubeError> {
        if limit == 0 {
            return Ok((Vec::new(), false));
        }

        let cooldown_active = self
            .quota_cooldown
            .as_ref()
            .is_some_and(|cd| cd.is_active());

        if let Some(api_key) = self.api_key.as_deref() {
            if cooldown_active {
                tracing::debug!(
                    channel_id = %channel_id,
                    "skipping YouTube Data API backfill due to active quota cooldown, using InnerTube"
                );
            } else {
                match self
                    .fetch_videos_from_data_api(channel_id, api_key, limit, known_video_ids, until)
                    .await
                {
                    Ok(result) => return Ok(result),
                    Err(err) => {
                        tracing::warn!(
                            channel_id = %channel_id,
                            error = %err,
                            "YouTube Data API backfill failed, falling back to InnerTube"
                        );
                    }
                }
            }
        }

        self.fetch_videos_backfill_missing_via_innertube(channel_id, known_video_ids, limit, until)
            .await
    }

    async fn fetch_videos_backfill_missing_via_innertube(
        &self,
        channel_id: &str,
        known_video_ids: &HashSet<String>,
        limit: usize,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<Video>, bool), YouTubeError> {
        let (mut current_ids, mut continuation) =
            self.fetch_innertube_initial_page(channel_id).await?;
        if current_ids.is_empty() && continuation.is_none() {
            return Ok((Vec::new(), true));
        }

        let mut total_videos = Vec::new();
        let mut total_selected_ids = Vec::new();
        let mut seen_in_this_run = HashSet::new();
        let mut pages_fetched = 0;
        let max_pages = if until.is_some() { 50 } else { 10 };

        loop {
            let next_batch_ids = current_ids
                .into_iter()
                .filter(|id| !seen_in_this_run.contains(id))
                .collect::<Vec<_>>();

            for id in next_batch_ids {
                seen_in_this_run.insert(id.clone());
                let is_missing = !known_video_ids.contains(&id);
                if is_missing || until.is_some() {
                    match self.fetch_watch_metadata(&id).await {
                        Ok(metadata) => {
                            let Some(pub_at) = metadata.published_at else {
                                tracing::warn!(video_id = %id, "metadata missing published_at during crawl, skipping video");
                                continue;
                            };

                            if let Some(until_date) = until {
                                if pub_at < until_date {
                                    continuation = None;
                                    break;
                                }
                            }

                            if is_missing && total_selected_ids.len() < limit {
                                tracing::debug!(
                                    channel_id = %channel_id,
                                    video_id = %id,
                                    title = %metadata.title,
                                    published_at = %pub_at.to_rfc3339(),
                                    "backfill: found missing video via InnerTube"
                                );
                                total_selected_ids.push(id.clone());
                                total_videos.push(Video {
                                    id: id.clone(),
                                    channel_id: channel_id.to_string(),
                                    title: metadata.title,
                                    thumbnail_url: metadata.thumbnail_url,
                                    published_at: pub_at,
                                    is_short: self.fetch_is_short_flag(&id).await,
                                    transcript_status: crate::models::ContentStatus::Pending,
                                    summary_status: crate::models::ContentStatus::Pending,
                                    acknowledged: false,
                                    retry_count: 0,
                                });
                            }
                        }
                        Err(err) => {
                            tracing::warn!(video_id = %id, error = %err, "failed to fetch metadata during crawl, skipping video to avoid date corruption");
                        }
                    }
                }

                if total_selected_ids.len() >= limit && until.is_none() {
                    break;
                }
            }

            if (total_selected_ids.len() >= limit && until.is_none())
                || continuation.is_none()
                || pages_fetched >= max_pages
            {
                break;
            }

            if let Some(token) = continuation {
                pages_fetched += 1;
                match self.fetch_continuation_page(&token).await {
                    Ok((next_ids, next_token)) => {
                        if next_ids.is_empty() && next_token.is_none() {
                            continuation = None;
                            break;
                        }
                        current_ids = next_ids;
                        continuation = next_token;
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "failed to fetch continuation page while backfilling");
                        continuation = None;
                        break;
                    }
                }
            } else {
                break;
            }
        }

        let exhausted = continuation.is_none();
        Ok((total_videos, exhausted))
    }

    async fn fetch_videos_from_data_api(
        &self,
        channel_id: &str,
        api_key: &str,
        limit: usize,
        known_video_ids: &HashSet<String>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<Video>, bool), YouTubeError> {
        let uploads_playlist_id = self
            .fetch_data_api_uploads_playlist_id(channel_id, api_key)
            .await?;

        let mut page_token: Option<String> = None;
        let mut videos = Vec::new();
        let mut seen = HashSet::new();
        let max_pages = if until.is_some() { 50 } else { 10 };
        let target_scan_count = limit
            .saturating_add(known_video_ids.len())
            .saturating_add(10);
        let mut pages_fetched = 0usize;

        let exhausted = loop {
            let mut request = self
                .client
                .get("https://www.googleapis.com/youtube/v3/playlistItems")
                .query(&[
                    ("part", "snippet"),
                    ("playlistId", uploads_playlist_id.as_str()),
                    ("maxResults", "50"),
                    ("key", api_key),
                ]);

            if let Some(token) = page_token.as_deref() {
                request = request.query(&[("pageToken", token)]);
            }

            let response = request.send().await?;
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                tracing::warn!(
                    channel_id = %channel_id,
                    status = %status,
                    body = %body,
                    "Data API playlistItems request failed"
                );
                if Self::is_quota_exceeded(&body) {
                    if let Some(cd) = &self.quota_cooldown {
                        cd.activate();
                    }
                }
                return Err(YouTubeError::ChannelNotFound);
            }

            let payload: DataApiListResponse<DataApiPlaylistItem> =
                response.json().await.map_err(YouTubeError::FetchError)?;
            page_token = payload.next_page_token;
            pages_fetched += 1;

            for item in payload.items.unwrap_or_default() {
                let Some(video_id) = item
                    .snippet
                    .resource_id
                    .as_ref()
                    .and_then(|rid| rid.video_id.as_deref())
                    .map(str::trim)
                    .filter(|id| id.len() == 11)
                    .map(ToOwned::to_owned)
                else {
                    continue;
                };

                if !seen.insert(video_id.clone()) {
                    continue;
                }
                if known_video_ids.contains(&video_id) {
                    continue;
                }

                let published_at = item
                    .snippet
                    .published_at
                    .as_deref()
                    .and_then(Self::parse_any_datetime);

                let Some(effective_published_at) = published_at else {
                    tracing::warn!(video_id = %video_id, "Data API snippet missing publishedAt, skipping video");
                    continue;
                };

                if let Some(until_date) = until {
                    if effective_published_at < until_date {
                        return Ok((videos, true));
                    }
                }

                if videos.len() >= limit {
                    continue;
                }

                tracing::debug!(
                    channel_id = %channel_id,
                    video_id = %video_id,
                    title = %item.snippet.title,
                    published_at = %effective_published_at.to_rfc3339(),
                    "data_api: found video"
                );

                let thumbnail_url =
                    Self::pick_data_api_thumbnail_url(item.snippet.thumbnails.as_ref());
                let is_short = self.fetch_is_short_flag(&video_id).await;
                videos.push(Video {
                    id: video_id,
                    channel_id: channel_id.to_string(),
                    title: item.snippet.title,
                    thumbnail_url,
                    published_at: effective_published_at,
                    is_short,
                    transcript_status: crate::models::ContentStatus::Pending,
                    summary_status: crate::models::ContentStatus::Pending,
                    acknowledged: false,
                    retry_count: 0,
                });

                if videos.len() >= limit && until.is_none() {
                    return Ok((videos, page_token.is_none()));
                }
            }

            if page_token.is_none() {
                break true;
            }
            if pages_fetched >= max_pages || seen.len() >= target_scan_count {
                break false;
            }
        };

        Ok((videos, exhausted))
    }

    async fn fetch_data_api_uploads_playlist_id(
        &self,
        channel_id: &str,
        api_key: &str,
    ) -> Result<String, YouTubeError> {
        let response = self
            .client
            .get("https://www.googleapis.com/youtube/v3/channels")
            .query(&[
                ("part", "contentDetails"),
                ("id", channel_id),
                ("maxResults", "1"),
                ("key", api_key),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!(
                channel_id = %channel_id,
                status = %status,
                body = %body,
                "Data API channels request failed"
            );
            if Self::is_quota_exceeded(&body) {
                if let Some(cd) = &self.quota_cooldown {
                    cd.activate();
                }
            }
            return Err(YouTubeError::ChannelNotFound);
        }

        let payload: DataApiListResponse<DataApiChannelItem> =
            response.json().await.map_err(YouTubeError::FetchError)?;
        payload
            .items
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|item| item.content_details.related_playlists.uploads)
            .filter(|uploads| !uploads.trim().is_empty())
            .ok_or(YouTubeError::ChannelNotFound)
    }

    fn pick_data_api_thumbnail_url(thumbnails: Option<&DataApiThumbnails>) -> Option<String> {
        let thumbs = thumbnails?;
        thumbs
            .maxres
            .as_ref()
            .and_then(|thumb| thumb.url.as_ref())
            .or_else(|| {
                thumbs
                    .standard
                    .as_ref()
                    .and_then(|thumb| thumb.url.as_ref())
            })
            .or_else(|| thumbs.high.as_ref().and_then(|thumb| thumb.url.as_ref()))
            .or_else(|| thumbs.medium.as_ref().and_then(|thumb| thumb.url.as_ref()))
            .or_else(|| thumbs.default.as_ref().and_then(|thumb| thumb.url.as_ref()))
            .map(ToOwned::to_owned)
    }

    async fn fetch_innertube_initial_page(
        &self,
        channel_id: &str,
    ) -> Result<(Vec<String>, Option<String>), YouTubeError> {
        let url = "https://www.youtube.com/youtubei/v1/browse?prettyPrint=false";
        let body = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20240225.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "browseId": channel_id,
            "params": "EgZ2aWRlb3PyBgQKAjoA"
        });

        let response = self
            .client
            .post(url)
            .header("User-Agent", Self::desktop_user_agent())
            .header("Referer", "https://www.youtube.com/")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            tracing::warn!(status = %status, body = %text, "InnerTube initial request failed");
            return Ok((Vec::new(), None));
        }

        let data: Value = response.json().await.map_err(YouTubeError::FetchError)?;
        let mut ids = Vec::new();
        let mut continuation = None;
        Self::extract_ids_and_continuation_from_value(&data, &mut ids, &mut continuation);
        Ok((ids, continuation))
    }

    fn extract_ids_and_continuation_from_value(
        value: &Value,
        ids: &mut Vec<String>,
        continuation: &mut Option<String>,
    ) {
        match value {
            Value::Object(map) => {
                // Look for video IDs
                if let Some(Value::String(id)) = map.get("videoId") {
                    if id.len() == 11 && !ids.contains(id) {
                        ids.push(id.clone());
                    }
                }

                // Look for continuation tokens in various common locations
                if let Some(Value::String(token)) = map.get("continuation") {
                    if token.len() > 20 && continuation.is_none() {
                        *continuation = Some(token.clone());
                    }
                }

                // Check continuationCommand token
                if continuation.is_none() {
                    if let Some(cmd) = map.get("continuationCommand") {
                        if let Some(Value::String(token)) = cmd.get("token") {
                            *continuation = Some(token.clone());
                        }
                    }
                }

                // Check nextContinuationData
                if continuation.is_none() {
                    if let Some(data) = map.get("nextContinuationData") {
                        if let Some(Value::String(token)) = data.get("continuation") {
                            *continuation = Some(token.clone());
                        }
                    }
                }

                // Check continuationItemRenderer
                if continuation.is_none() {
                    if let Some(renderer) = map.get("continuationItemRenderer") {
                        // Look for token anywhere inside this renderer
                        if let Some(token) = Self::find_token_in_value(renderer) {
                            *continuation = Some(token);
                        }
                    }
                }

                // Check reloadContinuationItemsCommand
                if continuation.is_none() {
                    if let Some(cmd) = map.get("reloadContinuationItemsCommand") {
                        if let Some(Value::String(token)) = cmd.get("token") {
                            *continuation = Some(token.clone());
                        }
                    }
                }

                for v in map.values() {
                    Self::extract_ids_and_continuation_from_value(v, ids, continuation);
                }
            }
            Value::Array(list) => {
                for v in list {
                    Self::extract_ids_and_continuation_from_value(v, ids, continuation);
                }
            }
            _ => {}
        }
    }

    fn find_token_in_value(value: &Value) -> Option<String> {
        match value {
            Value::Object(map) => {
                if let Some(Value::String(token)) = map.get("token") {
                    if token.len() > 20 {
                        return Some(token.clone());
                    }
                }
                if let Some(Value::String(token)) = map.get("continuation") {
                    if token.len() > 20 {
                        return Some(token.clone());
                    }
                }
                for v in map.values() {
                    if let Some(token) = Self::find_token_in_value(v) {
                        return Some(token);
                    }
                }
            }
            Value::Array(list) => {
                for v in list {
                    if let Some(token) = Self::find_token_in_value(v) {
                        return Some(token);
                    }
                }
            }
            _ => {}
        }
        None
    }

    async fn fetch_continuation_page(
        &self,
        token: &str,
    ) -> Result<(Vec<String>, Option<String>), YouTubeError> {
        // InnerTube API endpoint for browse
        let url = "https://www.youtube.com/youtubei/v1/browse?prettyPrint=false";

        // Minimal InnerTube context
        let body = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20240225.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "continuation": token
        });

        let response = self
            .client
            .post(url)
            .header("User-Agent", Self::desktop_user_agent())
            .header("Referer", "https://www.youtube.com/")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            tracing::warn!(status = %status, body = %text, "InnerTube continuation request failed");
            return Ok((Vec::new(), None));
        }

        let data: Value = response.json().await.map_err(YouTubeError::FetchError)?;
        let mut ids = Vec::new();
        let mut next_token = None;
        Self::extract_ids_and_continuation_from_value(&data, &mut ids, &mut next_token);

        Ok((ids, next_token))
    }

    async fn fetch_watch_metadata(&self, video_id: &str) -> Result<WatchMetadata, YouTubeError> {
        let watch_url = format!("https://www.youtube.com/watch?v={video_id}");
        tracing::debug!(video_id = %video_id, "fetching watch metadata via full page");

        let response = self
            .client
            .get(&watch_url)
            .header("User-Agent", Self::desktop_user_agent())
            .send()
            .await?;

        if !response.status().is_success() {
            tracing::warn!(video_id = %video_id, status = %response.status(), "watch page fetch failed");
            return Err(YouTubeError::ChannelNotFound);
        }

        let html = response.text().await?;
        let details = Self::extract_video_details_from_watch_html(&html);

        Ok(WatchMetadata {
            title: details
                .title
                .unwrap_or_else(|| format!("YouTube video {video_id}")),
            thumbnail_url: details.thumbnail_url,
            published_at: details.published_at,
        })
    }

    pub async fn fetch_video_info(&self, video_id: &str) -> Result<VideoInfo, YouTubeError> {
        let watch_url = format!("https://www.youtube.com/watch?v={video_id}");
        let response = self
            .client
            .get(&watch_url)
            .header("User-Agent", Self::desktop_user_agent())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(YouTubeError::ChannelNotFound);
        }

        let html = response.text().await?;
        let details = Self::extract_video_details_from_watch_html(&html);

        Ok(VideoInfo {
            video_id: video_id.to_string(),
            watch_url,
            title: details
                .title
                .unwrap_or_else(|| format!("YouTube video {video_id}")),
            description: details.description,
            thumbnail_url: details.thumbnail_url,
            channel_name: details.channel_name,
            channel_id: details.channel_id,
            published_at: details.published_at,
            duration_iso8601: details.duration_iso8601,
            duration_seconds: details.duration_seconds,
            view_count: details.view_count,
        })
    }

    fn extract_video_details_from_watch_html(html: &str) -> WatchVideoDetails {
        let document = Html::parse_document(html);
        let mut details = WatchVideoDetails {
            title: Self::extract_meta_content(&document, r#"meta[property="og:title"]"#)
                .map(ToOwned::to_owned),
            description: Self::extract_meta_content(
                &document,
                r#"meta[property="og:description"]"#,
            )
            .or_else(|| Self::extract_meta_content(&document, r#"meta[name="description"]"#))
            .map(ToOwned::to_owned),
            thumbnail_url: Self::extract_meta_content(&document, r#"meta[property="og:image"]"#)
                .map(ToOwned::to_owned),
            channel_name: Self::extract_meta_content(&document, r#"link[itemprop="name"]"#)
                .map(ToOwned::to_owned),
            channel_id: Self::extract_meta_content(&document, r#"meta[itemprop="channelId"]"#)
                .map(ToOwned::to_owned),
            published_at: Self::extract_meta_content(
                &document,
                r#"meta[itemprop="datePublished"]"#,
            )
            .and_then(Self::parse_any_datetime),
            view_count: Self::extract_meta_content(
                &document,
                r#"meta[itemprop="interactionCount"]"#,
            )
            .and_then(Self::parse_u64),
            ..Default::default()
        };

        Self::merge_ld_json_video_details(&document, &mut details);
        details
    }

    fn extract_meta_content<'a>(document: &'a Html, selector: &str) -> Option<&'a str> {
        let selector = Selector::parse(selector).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|node| {
                node.value()
                    .attr("content")
                    .or_else(|| node.value().attr("href"))
            })
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    fn merge_ld_json_video_details(document: &Html, details: &mut WatchVideoDetails) {
        let selector = match Selector::parse(r#"script[type="application/ld+json"]"#) {
            Ok(sel) => sel,
            Err(_) => return,
        };

        for script in document.select(&selector) {
            let raw = script.inner_html();
            let parsed = match serde_json::from_str::<Value>(&raw) {
                Ok(value) => value,
                Err(_) => continue,
            };
            Self::fill_from_ld_json_value(&parsed, details);
        }
    }

    fn fill_from_ld_json_value(value: &Value, details: &mut WatchVideoDetails) {
        match value {
            Value::Array(items) => {
                for item in items {
                    Self::fill_from_ld_json_value(item, details);
                }
            }
            Value::Object(map) => {
                let is_video_object = map
                    .get("@type")
                    .map(Self::value_has_video_object_type)
                    .unwrap_or(false);

                if is_video_object {
                    if details.title.is_none() {
                        details.title = map
                            .get("name")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(ToOwned::to_owned);
                    }
                    if details.description.is_none() {
                        details.description = map
                            .get("description")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(ToOwned::to_owned);
                    }
                    if details.thumbnail_url.is_none() {
                        details.thumbnail_url = map
                            .get("thumbnailUrl")
                            .and_then(Self::extract_string_or_first_string)
                            .map(ToOwned::to_owned);
                    }
                    if details.channel_name.is_none() {
                        details.channel_name = map
                            .get("author")
                            .and_then(Self::extract_author_name)
                            .map(ToOwned::to_owned);
                    }
                    if details.published_at.is_none() {
                        details.published_at = map
                            .get("uploadDate")
                            .and_then(Value::as_str)
                            .or_else(|| map.get("datePublished").and_then(Value::as_str))
                            .and_then(Self::parse_any_datetime);
                    }
                    if details.duration_iso8601.is_none() {
                        details.duration_iso8601 = map
                            .get("duration")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(ToOwned::to_owned);
                    }
                    if details.duration_seconds.is_none() {
                        details.duration_seconds = details
                            .duration_iso8601
                            .as_deref()
                            .and_then(Self::parse_iso8601_duration_seconds);
                    }
                    if details.view_count.is_none() {
                        details.view_count = map
                            .get("interactionCount")
                            .and_then(Self::extract_u64_from_value)
                            .or_else(|| {
                                map.get("interactionStatistic")
                                    .and_then(Self::extract_interaction_view_count)
                            });
                    }
                    if details.channel_id.is_none() {
                        details.channel_id = map
                            .get("channelId")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(ToOwned::to_owned);
                    }
                }
            }
            _ => {}
        }
    }

    fn value_has_video_object_type(value: &Value) -> bool {
        match value {
            Value::String(single) => single.eq_ignore_ascii_case("VideoObject"),
            Value::Array(items) => items.iter().any(Self::value_has_video_object_type),
            _ => false,
        }
    }

    fn extract_author_name(value: &Value) -> Option<&str> {
        match value {
            Value::Object(map) => map
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty()),
            Value::Array(items) => items.iter().find_map(Self::extract_author_name),
            _ => None,
        }
    }

    fn extract_string_or_first_string(value: &Value) -> Option<&str> {
        match value {
            Value::String(single) => {
                let trimmed = single.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }
            Value::Array(items) => items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .find(|text| !text.is_empty()),
            _ => None,
        }
    }

    fn extract_u64_from_value(value: &Value) -> Option<u64> {
        match value {
            Value::Number(number) => number.as_u64(),
            Value::String(text) => Self::parse_u64(text),
            _ => None,
        }
    }

    fn extract_interaction_view_count(value: &Value) -> Option<u64> {
        match value {
            Value::Object(map) => {
                let interaction_type = map
                    .get("interactionType")
                    .and_then(Value::as_object)
                    .and_then(|inner| inner.get("@type"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();

                if interaction_type == "http://schema.org/WatchAction"
                    || interaction_type == "WatchAction"
                {
                    return map
                        .get("userInteractionCount")
                        .and_then(Self::extract_u64_from_value);
                }

                map.get("userInteractionCount")
                    .and_then(Self::extract_u64_from_value)
            }
            Value::Array(items) => items.iter().find_map(Self::extract_interaction_view_count),
            _ => None,
        }
    }

    fn parse_u64(value: &str) -> Option<u64> {
        let digits = value
            .chars()
            .filter(|ch| ch.is_ascii_digit())
            .collect::<String>();
        if digits.is_empty() {
            return None;
        }
        digits.parse::<u64>().ok()
    }

    fn parse_any_datetime(value: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .ok()
            .or_else(|| Self::parse_yyyy_mm_dd(value))
    }

    fn parse_iso8601_duration_seconds(value: &str) -> Option<u64> {
        if !value.starts_with('P') {
            return None;
        }

        let mut total = 0u64;
        let mut in_time = false;
        let mut current = String::new();
        let mut matched_unit = false;

        for ch in value.chars().skip(1) {
            if ch == 'T' {
                in_time = true;
                continue;
            }
            if ch.is_ascii_digit() {
                current.push(ch);
                continue;
            }
            if current.is_empty() {
                continue;
            }

            let amount = current.parse::<u64>().ok()?;
            match ch {
                'D' => total = total.saturating_add(amount.saturating_mul(86_400)),
                'H' => total = total.saturating_add(amount.saturating_mul(3_600)),
                'M' if in_time => total = total.saturating_add(amount.saturating_mul(60)),
                'S' => total = total.saturating_add(amount),
                _ => return None,
            }
            current.clear();
            matched_unit = true;
        }

        if matched_unit { Some(total) } else { None }
    }

    fn parse_videos_from_feed(
        content: &[u8],
        channel_id: &str,
    ) -> Result<Vec<Video>, YouTubeError> {
        if let Ok(channel) = rss::Channel::read_from(content) {
            let videos = Self::videos_from_rss_channel(&channel, channel_id);
            return Ok(videos);
        }

        Self::videos_from_atom_feed(content, channel_id)
    }

    fn videos_from_rss_channel(channel: &rss::Channel, channel_id: &str) -> Vec<Video> {
        channel
            .items()
            .iter()
            .filter_map(|item| {
                let video_id = item
                    .link()
                    .and_then(Self::extract_video_id_from_url)
                    .or_else(|| {
                        // YouTube RSS uses yt:videoId extension
                        item.extensions()
                            .get("yt")
                            .and_then(|yt| yt.get("videoId"))
                            .and_then(|v| v.first())
                            .and_then(|ext| ext.value.as_deref())
                            .map(str::trim)
                            .filter(|id| !id.is_empty())
                            .map(ToOwned::to_owned)
                    })?;

                let published = item
                    .pub_date()
                    .and_then(|d| chrono::DateTime::parse_from_rfc2822(d).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))?;

                tracing::debug!(
                    channel_id = %channel_id,
                    video_id = %video_id,
                    title = %item.title().unwrap_or("Untitled"),
                    published_at = %published.to_rfc3339(),
                    "feed: found video via RSS"
                );

                // Extract thumbnail from media:group
                let thumbnail = item
                    .extensions()
                    .get("media")
                    .and_then(|media| media.get("group"))
                    .and_then(|g| g.first())
                    .and_then(|group| group.children.get("thumbnail"))
                    .and_then(|thumbs| thumbs.first())
                    .and_then(|t| t.attrs.get("url"))
                    .cloned();

                Some(Video {
                    id: video_id,
                    channel_id: channel_id.to_string(),
                    title: item.title().unwrap_or("Untitled").to_string(),
                    thumbnail_url: thumbnail,
                    published_at: published,
                    is_short: false,
                    transcript_status: crate::models::ContentStatus::Pending,
                    summary_status: crate::models::ContentStatus::Pending,
                    acknowledged: false,
                    retry_count: 0,
                })
            })
            .collect()
    }

    fn videos_from_atom_feed(content: &[u8], channel_id: &str) -> Result<Vec<Video>, YouTubeError> {
        let xml = std::str::from_utf8(content)
            .map_err(|e| YouTubeError::FeedParseError(format!("invalid UTF-8 response: {e}")))?;
        let document = roxmltree::Document::parse(xml)
            .map_err(|e| YouTubeError::FeedParseError(format!("invalid XML response: {e}")))?;
        let root = document.root_element();
        if root.tag_name().name() != "feed" {
            return Err(YouTubeError::FeedParseError(
                "XML root element is not an Atom <feed>".to_string(),
            ));
        }

        let videos = root
            .children()
            .filter(|node| node.is_element() && node.tag_name().name() == "entry")
            .filter_map(|entry| {
                let video_id = entry
                    .descendants()
                    .find(|node| node.is_element() && node.tag_name().name() == "videoId")
                    .and_then(|node| node.text())
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .map(ToOwned::to_owned)
                    .or_else(|| {
                        entry
                            .children()
                            .find(|node| node.is_element() && node.tag_name().name() == "link")
                            .and_then(|node| node.attribute("href"))
                            .and_then(Self::extract_video_id_from_url)
                    })
                    .or_else(|| {
                        entry
                            .children()
                            .find(|node| node.is_element() && node.tag_name().name() == "id")
                            .and_then(|node| node.text())
                            .and_then(Self::extract_video_id_from_atom_id)
                    })?;

                let title = entry
                    .children()
                    .find(|node| node.is_element() && node.tag_name().name() == "title")
                    .and_then(|node| node.text())
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .unwrap_or("Untitled")
                    .to_string();

                let published = entry
                    .children()
                    .find(|node| node.is_element() && node.tag_name().name() == "published")
                    .or_else(|| {
                        entry
                            .children()
                            .find(|node| node.is_element() && node.tag_name().name() == "updated")
                    })
                    .and_then(|node| node.text())
                    .and_then(|text| chrono::DateTime::parse_from_rfc3339(text).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                let thumbnail_url = entry
                    .descendants()
                    .find(|node| node.is_element() && node.tag_name().name() == "thumbnail")
                    .and_then(|node| node.attribute("url"))
                    .map(ToOwned::to_owned);

                let published_at = published?;

                tracing::debug!(
                    channel_id = %channel_id,
                    video_id = %video_id,
                    title = %title,
                    published_at = %published_at.to_rfc3339(),
                    "feed: found video via Atom"
                );

                Some(Video {
                    id: video_id,
                    channel_id: channel_id.to_string(),
                    title,
                    thumbnail_url,
                    published_at,
                    is_short: false,
                    transcript_status: crate::models::ContentStatus::Pending,
                    summary_status: crate::models::ContentStatus::Pending,
                    acknowledged: false,
                    retry_count: 0,
                })
            })
            .collect();

        Ok(videos)
    }

    fn extract_video_id_from_url(link: &str) -> Option<String> {
        link.split("v=")
            .nth(1)
            .map(|id| id.split('&').next().unwrap_or(id).to_string())
            .filter(|id| !id.is_empty())
    }

    fn extract_video_id_from_atom_id(entry_id: &str) -> Option<String> {
        entry_id
            .trim()
            .strip_prefix("yt:video:")
            .map(ToOwned::to_owned)
            .filter(|id| !id.is_empty())
    }

    fn parse_yyyy_mm_dd(input: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d")
            .ok()?
            .and_hms_opt(0, 0, 0)?
            .and_utc()
            .into()
    }

    async fn fetch_is_short_flag(&self, video_id: &str) -> bool {
        let shorts_url = format!("https://www.youtube.com/shorts/{video_id}");
        match self.client.head(&shorts_url).send().await {
            Ok(response) if response.status().is_success() => {
                Self::is_short_from_resolved_url(response.url().as_str())
            }
            _ => false,
        }
    }

    fn is_short_from_resolved_url(url: &str) -> bool {
        reqwest::Url::parse(url)
            .map(|parsed| parsed.path().starts_with("/shorts/"))
            .unwrap_or_else(|_| url.contains("/shorts/"))
    }
}

impl Default for YouTubeService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::YouTubeService;

    #[test]
    fn test_channel_id_detection() {
        // Valid channel IDs
        assert!("UCBcRF18a7Qf58cCRy5xuWwQ".starts_with("UC"));
        assert_eq!("UCBcRF18a7Qf58cCRy5xuWwQ".len(), 24);
    }

    #[test]
    fn test_parse_rss_feed_videos() {
        let feed = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:yt="http://www.youtube.com/xml/schemas/2015" xmlns:media="http://search.yahoo.com/mrss/">
  <channel>
    <item>
      <title>Example Video</title>
      <link>https://www.youtube.com/watch?v=abc123xyz99</link>
      <pubDate>Thu, 26 Feb 2026 10:00:00 +0000</pubDate>
      <media:group>
        <media:thumbnail url="https://img.example.com/thumb.jpg"/>
      </media:group>
      <yt:videoId>abc123xyz99</yt:videoId>
    </item>
  </channel>
</rss>"#;

        let videos = YouTubeService::parse_videos_from_feed(feed.as_bytes(), "UC_TEST")
            .expect("RSS feed should parse");

        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].id, "abc123xyz99");
        assert_eq!(videos[0].title, "Example Video");
        assert_eq!(videos[0].channel_id, "UC_TEST");
        assert_eq!(
            videos[0].thumbnail_url.as_deref(),
            Some("https://img.example.com/thumb.jpg")
        );
    }

    #[test]
    fn test_parse_atom_feed_videos() {
        let feed = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:yt="http://www.youtube.com/xml/schemas/2015" xmlns:media="http://search.yahoo.com/mrss/">
  <entry>
    <yt:videoId>def456uvw00</yt:videoId>
    <title>Atom Video</title>
    <published>2026-02-26T12:30:00+00:00</published>
    <link rel="alternate" href="https://www.youtube.com/watch?v=def456uvw00"/>
    <media:group>
      <media:thumbnail url="https://img.example.com/atom-thumb.jpg" width="196" height="110"/>
    </media:group>
  </entry>
</feed>"#;

        let videos = YouTubeService::parse_videos_from_feed(feed.as_bytes(), "UC_TEST")
            .expect("Atom feed should parse");

        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].id, "def456uvw00");
        assert_eq!(videos[0].title, "Atom Video");
        assert_eq!(videos[0].channel_id, "UC_TEST");
        assert_eq!(
            videos[0].thumbnail_url.as_deref(),
            Some("https://img.example.com/atom-thumb.jpg")
        );
    }

    #[test]
    fn test_parse_rss_feed_videos_missing_date() {
        let feed = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:yt="http://www.youtube.com/xml/schemas/2015">
  <channel>
    <item>
      <title>No Date Video</title>
      <link>https://www.youtube.com/watch?v=nodate12345</link>
      <yt:videoId>nodate12345</yt:videoId>
    </item>
  </channel>
</rss>"#;

        let videos = YouTubeService::parse_videos_from_feed(feed.as_bytes(), "UC_TEST")
            .expect("RSS feed should parse");

        assert_eq!(videos.len(), 0, "Videos without pubDate should be skipped");
    }

    #[test]
    fn test_parse_atom_feed_videos_missing_date() {
        let feed = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:yt="http://www.youtube.com/xml/schemas/2015">
  <entry>
    <yt:videoId>nodate67890</yt:videoId>
    <title>No Date Atom Video</title>
    <link rel="alternate" href="https://www.youtube.com/watch?v=nodate67890"/>
  </entry>
</feed>"#;

        let videos = YouTubeService::parse_videos_from_feed(feed.as_bytes(), "UC_TEST")
            .expect("Atom feed should parse");

        assert_eq!(
            videos.len(),
            0,
            "Atom entries without published/updated should be skipped"
        );
    }

    #[test]
    fn test_extract_channel_thumbnail_from_itemprop() {
        let html = r#"
<html>
  <head>
    <link itemprop="thumbnailUrl" href="https://yt3.googleusercontent.com/channel-avatar=s900" />
  </head>
</html>
"#;
        let document = scraper::Html::parse_document(html);
        let service = YouTubeService::new();

        assert_eq!(
            service
                .extract_channel_thumbnail(&document, html)
                .as_deref(),
            Some("https://yt3.googleusercontent.com/channel-avatar=s900")
        );
    }

    #[test]
    fn test_extract_channel_thumbnail_from_image_src() {
        let html = r#"
<html>
  <head>
    <link rel="image_src" href="https://yt3.googleusercontent.com/channel-avatar=s900" />
  </head>
</html>
"#;
        let document = scraper::Html::parse_document(html);
        let service = YouTubeService::new();

        assert_eq!(
            service
                .extract_channel_thumbnail(&document, html)
                .as_deref(),
            Some("https://yt3.googleusercontent.com/channel-avatar=s900")
        );
    }

    #[test]
    fn test_extract_channel_thumbnail_from_embedded_data() {
        let html = r#"
<html>
  <head></head>
  <body>
    <script>
      var ytInitialData = {"metadata":{"channelMetadataRenderer":{"avatar":{"thumbnails":[{"url":"https://yt3.googleusercontent.com/channel-avatar=s900\\u0026c=1"}]}}}};
    </script>
  </body>
</html>
"#;
        let document = scraper::Html::parse_document(html);
        let service = YouTubeService::new();

        assert_eq!(
            service
                .extract_channel_thumbnail(&document, html)
                .as_deref(),
            Some("https://yt3.googleusercontent.com/channel-avatar=s900&c=1")
        );
    }

    #[test]
    fn test_is_short_from_resolved_url() {
        assert!(YouTubeService::is_short_from_resolved_url(
            "https://www.youtube.com/shorts/abc123"
        ));
        assert!(!YouTubeService::is_short_from_resolved_url(
            "https://www.youtube.com/watch?v=abc123"
        ));
    }

    #[test]
    fn test_extract_video_details_from_watch_html() {
        let html = r#"
<html>
  <head>
    <meta property="og:title" content="Full Video Title" />
    <meta property="og:description" content="Full description text" />
    <meta property="og:image" content="https://img.example.com/full.jpg" />
    <meta itemprop="channelId" content="UC1234567890123456789012" />
    <meta itemprop="datePublished" content="2026-01-15" />
    <meta itemprop="interactionCount" content="12345" />
    <script type="application/ld+json">
      {
        "@context": "https://schema.org",
        "@type": "VideoObject",
        "name": "Full Video Title",
        "description": "Full description text",
        "uploadDate": "2026-01-15",
        "duration": "PT1H2M3S",
        "author": {"@type": "Person", "name": "Channel Name"}
      }
    </script>
  </head>
</html>
"#;

        let details = YouTubeService::extract_video_details_from_watch_html(html);
        assert_eq!(details.title.as_deref(), Some("Full Video Title"));
        assert_eq!(
            details.description.as_deref(),
            Some("Full description text")
        );
        assert_eq!(
            details.thumbnail_url.as_deref(),
            Some("https://img.example.com/full.jpg")
        );
        assert_eq!(
            details.channel_id.as_deref(),
            Some("UC1234567890123456789012")
        );
        assert_eq!(details.channel_name.as_deref(), Some("Channel Name"));
        assert_eq!(details.view_count, Some(12_345));
        assert_eq!(details.duration_iso8601.as_deref(), Some("PT1H2M3S"));
        assert_eq!(details.duration_seconds, Some(3_723));
    }

    #[test]
    fn test_parse_iso8601_duration_seconds() {
        assert_eq!(
            YouTubeService::parse_iso8601_duration_seconds("PT4M13S"),
            Some(253)
        );
        assert_eq!(
            YouTubeService::parse_iso8601_duration_seconds("PT1H2M3S"),
            Some(3723)
        );
        assert_eq!(
            YouTubeService::parse_iso8601_duration_seconds("P2DT1H"),
            Some(176_400)
        );
        assert_eq!(YouTubeService::parse_iso8601_duration_seconds("abc"), None);
    }
}
