use std::collections::HashSet;

use serde_json::Value;

use crate::models::Video;

use super::video_builder::build_pending_video;
use super::{YouTubeError, YouTubeService};

impl YouTubeService {
    pub(crate) async fn fetch_videos_backfill_missing_via_innertube(
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
                                total_videos.push(build_pending_video(
                                    channel_id,
                                    id.clone(),
                                    metadata.title,
                                    metadata.thumbnail_url,
                                    pub_at,
                                    self.fetch_is_short_flag(&id).await,
                                ));
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

    pub(crate) async fn fetch_is_short_flag(&self, video_id: &str) -> bool {
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
