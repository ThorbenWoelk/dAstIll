use scraper::{Html, Selector};
use serde_json::Value;

use crate::models::VideoInfo;

use super::{WatchMetadata, WatchVideoDetails, YouTubeError, YouTubeService};

impl YouTubeService {
    pub(crate) async fn resolve_from_url(
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

    pub(crate) async fn resolve_from_handle(
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

    pub(crate) async fn fetch_channel_info(
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

    pub(crate) async fn fetch_watch_metadata(
        &self,
        video_id: &str,
    ) -> Result<WatchMetadata, YouTubeError> {
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
        Self::merge_player_response_video_details(html, &mut details);
        details
    }

    fn merge_player_response_video_details(html: &str, details: &mut WatchVideoDetails) {
        let Some(player_response) = Self::extract_json_assignment(html, "ytInitialPlayerResponse")
            .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        else {
            return;
        };

        if details.title.is_none() {
            details.title = Self::value_at_path(&player_response, &["videoDetails", "title"])
                .and_then(Self::extract_string_or_first_string)
                .map(ToOwned::to_owned);
        }
        if details.channel_name.is_none() {
            details.channel_name =
                Self::value_at_path(&player_response, &["videoDetails", "author"])
                    .or_else(|| {
                        Self::value_at_path(&player_response, &["videoDetails", "ownerChannelName"])
                    })
                    .and_then(Self::extract_string_or_first_string)
                    .map(ToOwned::to_owned);
        }
        if details.channel_id.is_none() {
            details.channel_id =
                Self::value_at_path(&player_response, &["videoDetails", "externalChannelId"])
                    .and_then(Self::extract_string_or_first_string)
                    .map(ToOwned::to_owned);
        }
        if details.published_at.is_none() {
            details.published_at = Self::value_at_path(
                &player_response,
                &["microformat", "playerMicroformatRenderer", "publishDate"],
            )
            .or_else(|| {
                Self::value_at_path(
                    &player_response,
                    &["microformat", "playerMicroformatRenderer", "uploadDate"],
                )
            })
            .and_then(Self::extract_string_or_first_string)
            .and_then(Self::parse_any_datetime);
        }
        if details.duration_seconds.is_none() {
            details.duration_seconds =
                Self::value_at_path(&player_response, &["videoDetails", "lengthSeconds"])
                    .and_then(Self::extract_u64_from_value)
                    .or_else(|| {
                        Self::value_at_path(&player_response, &["videoDetails", "approxDurationMs"])
                            .and_then(Self::extract_u64_from_value)
                            .map(|milliseconds| milliseconds / 1_000)
                    });
        }
        if details.view_count.is_none() {
            details.view_count =
                Self::value_at_path(&player_response, &["videoDetails", "viewCount"])
                    .and_then(Self::extract_u64_from_value);
        }
    }

    fn extract_json_assignment<'a>(html: &'a str, variable_name: &str) -> Option<&'a str> {
        let variable_offset = html.find(variable_name)?;
        let assignment = &html[variable_offset + variable_name.len()..];
        let json_start = variable_offset + variable_name.len() + assignment.find('{')?;
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;

        for (offset, ch) in html[json_start..].char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                    continue;
                }

                match ch {
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            match ch {
                '"' => in_string = true,
                '{' => depth = depth.saturating_add(1),
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let json_end = json_start + offset + ch.len_utf8();
                        return Some(&html[json_start..json_end]);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
        let mut current = value;
        for segment in path {
            current = current.get(*segment)?;
        }
        Some(current)
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

    pub(crate) fn parse_any_datetime(value: &str) -> Option<chrono::DateTime<chrono::Utc>> {
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
}
