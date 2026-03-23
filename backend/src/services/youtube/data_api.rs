use std::collections::HashSet;

use serde::Deserialize;

use crate::models::{Video, VideoInfo};

use super::video_builder::build_pending_video;
use super::{YouTubeError, YouTubeService};

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

#[derive(Deserialize)]
struct DataApiVideoItem {
    snippet: Option<DataApiVideoSnippet>,
    #[serde(rename = "contentDetails")]
    content_details: Option<DataApiVideoContentDetails>,
    statistics: Option<DataApiVideoStatistics>,
}

#[derive(Deserialize)]
struct DataApiVideoSnippet {
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "channelTitle")]
    channel_title: Option<String>,
    #[serde(rename = "channelId")]
    channel_id: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    thumbnails: Option<DataApiThumbnails>,
}

#[derive(Deserialize)]
struct DataApiVideoContentDetails {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct DataApiVideoStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
}

impl YouTubeService {
    pub(crate) async fn fetch_video_info_from_data_api(
        &self,
        video_id: &str,
        api_key: &str,
    ) -> Result<VideoInfo, YouTubeError> {
        let response = self
            .client
            .get("https://www.googleapis.com/youtube/v3/videos")
            .query(&[
                ("part", "snippet,contentDetails,statistics"),
                ("id", video_id),
                ("maxResults", "1"),
                ("key", api_key),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!(
                video_id = %video_id,
                status = %status,
                body = %body,
                "Data API video info request failed"
            );
            if Self::is_quota_exceeded(&body) {
                if let Some(cd) = &self.quota_cooldown {
                    cd.activate();
                }
            }
            return Err(YouTubeError::ChannelNotFound);
        }

        let payload: DataApiListResponse<DataApiVideoItem> =
            response.json().await.map_err(YouTubeError::FetchError)?;
        let item = payload
            .items
            .unwrap_or_default()
            .into_iter()
            .next()
            .ok_or(YouTubeError::ChannelNotFound)?;

        let snippet = item.snippet.ok_or(YouTubeError::ChannelNotFound)?;
        let content_details = item.content_details;
        let statistics = item.statistics;
        let duration_iso8601 = content_details.and_then(|details| details.duration);

        Ok(VideoInfo {
            video_id: video_id.to_string(),
            watch_url: format!("https://www.youtube.com/watch?v={video_id}"),
            title: snippet
                .title
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| format!("YouTube video {video_id}")),
            description: crate::services::youtube::placeholder::sanitize_optional_description(
                snippet.description,
            ),
            thumbnail_url: Self::pick_data_api_thumbnail_url(snippet.thumbnails.as_ref()),
            channel_name: snippet.channel_title.filter(|name| !name.trim().is_empty()),
            channel_id: snippet.channel_id.filter(|id| !id.trim().is_empty()),
            published_at: snippet
                .published_at
                .as_deref()
                .and_then(Self::parse_any_datetime),
            duration_seconds: duration_iso8601
                .as_deref()
                .and_then(parse_iso8601_duration_seconds),
            duration_iso8601,
            view_count: statistics
                .and_then(|stats| stats.view_count)
                .as_deref()
                .and_then(parse_u64),
        })
    }

    pub(crate) async fn fetch_videos_from_data_api(
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
                videos.push(build_pending_video(
                    channel_id,
                    video_id,
                    item.snippet.title,
                    thumbnail_url,
                    effective_published_at,
                    is_short,
                ));

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
