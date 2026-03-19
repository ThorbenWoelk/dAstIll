use chrono::{DateTime, Utc};

use crate::models::{ContentStatus, Video};

pub(super) fn build_pending_video(
    channel_id: &str,
    id: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: DateTime<Utc>,
    is_short: bool,
) -> Video {
    Video {
        id,
        channel_id: channel_id.to_string(),
        title,
        thumbnail_url,
        published_at,
        is_short,
        transcript_status: ContentStatus::Pending,
        summary_status: ContentStatus::Pending,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    }
}
