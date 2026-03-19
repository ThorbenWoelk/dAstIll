use crate::models::Video;

use super::video_builder::build_pending_video;
use super::{YouTubeError, YouTubeService};

impl YouTubeService {
    pub(crate) fn parse_videos_from_feed(
        content: &[u8],
        channel_id: &str,
    ) -> Result<Vec<Video>, YouTubeError> {
        if let Ok(channel) = ::rss::Channel::read_from(content) {
            let videos = Self::videos_from_rss_channel(&channel, channel_id);
            return Ok(videos);
        }

        Self::videos_from_atom_feed(content, channel_id)
    }

    fn videos_from_rss_channel(channel: &::rss::Channel, channel_id: &str) -> Vec<Video> {
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

                Some(build_pending_video(
                    channel_id,
                    video_id,
                    item.title().unwrap_or("Untitled").to_string(),
                    thumbnail,
                    published,
                    false,
                ))
            })
            .collect()
    }
}
