use crate::models::Video;

use super::video_builder::build_pending_video;
use super::{YouTubeError, YouTubeService};

impl YouTubeService {
    pub(crate) fn videos_from_atom_feed(
        content: &[u8],
        channel_id: &str,
    ) -> Result<Vec<Video>, YouTubeError> {
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

                Some(build_pending_video(
                    channel_id,
                    video_id,
                    title,
                    thumbnail_url,
                    published_at,
                    false,
                ))
            })
            .collect();

        Ok(videos)
    }

    pub(crate) fn extract_video_id_from_url(link: &str) -> Option<String> {
        link.split("v=")
            .nth(1)
            .map(|id| id.split('&').next().unwrap_or(id).to_string())
            .filter(|id| !id.is_empty())
    }

    pub(crate) fn extract_video_id_from_atom_id(entry_id: &str) -> Option<String> {
        entry_id
            .trim()
            .strip_prefix("yt:video:")
            .map(ToOwned::to_owned)
            .filter(|id| !id.is_empty())
    }

    pub(crate) fn parse_yyyy_mm_dd(input: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d")
            .ok()?
            .and_hms_opt(0, 0, 0)?
            .and_utc()
            .into()
    }
}
