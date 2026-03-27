use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;

use crate::models::Video;
use crate::services::build_http_client;
use crate::services::http::YouTubeQuotaCooldown;

mod atom;
mod data_api;
mod innertube;
pub mod placeholder;
mod resolve;
mod rss;
pub(super) mod video_builder;

#[derive(Error, Debug)]
pub enum YouTubeError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Failed to parse RSS: {0}")]
    RssError(#[from] ::rss::Error),
    #[error("Failed to parse feed: {0}")]
    FeedParseError(String),
    #[error("Channel not found")]
    ChannelNotFound,
    #[error("YouTube rate limit exceeded: {0}")]
    RateLimited(String),
    #[error("Invalid input format")]
    InvalidInput,
}

pub struct YouTubeService {
    client: Client,
    api_key: Option<String>,
    quota_cooldown: Option<Arc<YouTubeQuotaCooldown>>,
}

pub(crate) struct WatchMetadata {
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

impl YouTubeService {
    fn desktop_user_agent() -> &'static str {
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
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
}

impl Default for YouTubeService {
    fn default() -> Self {
        Self::new()
    }
}
