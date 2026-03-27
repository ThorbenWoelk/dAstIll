use chrono::{DateTime, Utc};

use crate::db;
use crate::models::{ContentStatus, Video};
use crate::services::search::SearchSourceKind;

use super::constants::CHAT_RECENT_ACTIVITY_VIDEO_LIMIT;
use super::tools::RecentLibraryActivityQuery;

#[derive(Debug, Clone)]
pub(crate) struct RecentLibraryActivityResult {
    pub(crate) summary: String,
    pub(crate) output: String,
    pub(crate) materials: Vec<db::SearchMaterial>,
}

#[derive(Debug, Clone)]
struct RecentActivityRow {
    video: Video,
    material: db::SearchMaterial,
}

pub(crate) fn is_recent_activity_query(prompt: &str) -> bool {
    let normalized = normalize_text(prompt);
    recent_markers()
        .iter()
        .any(|marker| normalized.contains(marker))
}

pub(crate) fn is_explicit_realtime_status_query(prompt: &str) -> bool {
    let normalized = normalize_text(prompt);
    normalized.contains(" live right now")
        || normalized.contains(" streaming right now")
        || normalized.contains(" outside youtube")
        || normalized.contains(" off platform")
        || normalized.contains(" announced anything")
        || normalized.contains(" posted today")
        || normalized.contains(" post today")
        || normalized.contains(" working on this week")
}

pub(crate) async fn execute_recent_library_activity_query(
    store: &db::Store,
    query: &RecentLibraryActivityQuery,
) -> Result<RecentLibraryActivityResult, String> {
    let channel_id = query
        .channel_id
        .as_deref()
        .ok_or_else(|| "recent_library_activity requires a resolved channel scope".to_string())?;
    let channel = db::get_channel(store, channel_id)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("channel `{channel_id}` not found"))?;

    let mut videos = db::load_all_videos(store)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|video| video.channel_id == channel.id)
        .collect::<Vec<_>>();
    videos.sort_by(|left, right| right.published_at.cmp(&left.published_at));

    let recent_total = videos.len();
    let limit_videos = query
        .limit_videos
        .clamp(1, CHAT_RECENT_ACTIVITY_VIDEO_LIMIT.max(1));

    let ready_videos = videos
        .iter()
        .filter(|video| {
            video.summary_status == ContentStatus::Ready
                || video.transcript_status == ContentStatus::Ready
        })
        .take(limit_videos)
        .cloned()
        .collect::<Vec<_>>();
    let unprocessed_recent = videos
        .iter()
        .take(limit_videos)
        .filter(|video| {
            video.summary_status != ContentStatus::Ready
                && video.transcript_status != ContentStatus::Ready
        })
        .count();

    let mut rows = Vec::new();
    for video in ready_videos {
        if let Some(material) = load_preferred_material(store, &video, query).await? {
            rows.push(RecentActivityRow { video, material });
        }
    }

    let summary = format!(
        "Review recent library activity for {} (latest {} processed video{})",
        channel.name,
        rows.len(),
        if rows.len() == 1 { "" } else { "s" }
    );

    if rows.is_empty() {
        let output = if recent_total == 0 {
            format!("{} is not present in the library yet.", channel.name)
        } else {
            format!(
                "Recent library activity for {} could not be reviewed because none of the latest {} videos are fully processed yet. This does not say anything about real-time off-platform activity.",
                channel.name,
                limit_videos.min(recent_total)
            )
        };
        return Ok(RecentLibraryActivityResult {
            summary,
            output,
            materials: Vec::new(),
        });
    }

    let output =
        format_recent_activity_output(&channel.name, limit_videos, &rows, unprocessed_recent);

    Ok(RecentLibraryActivityResult {
        summary,
        output,
        materials: rows.into_iter().map(|row| row.material).collect(),
    })
}

async fn load_preferred_material(
    store: &db::Store,
    video: &Video,
    query: &RecentLibraryActivityQuery,
) -> Result<Option<db::SearchMaterial>, String> {
    if query.include_summaries && video.summary_status == ContentStatus::Ready {
        if let Some(material) =
            db::load_search_material(store, &video.id, SearchSourceKind::Summary)
                .await
                .map_err(|error| error.to_string())?
        {
            return Ok(Some(material));
        }
    }

    if query.include_transcripts && video.transcript_status == ContentStatus::Ready {
        if let Some(material) =
            db::load_search_material(store, &video.id, SearchSourceKind::Transcript)
                .await
                .map_err(|error| error.to_string())?
        {
            return Ok(Some(material));
        }
    }

    Ok(None)
}

fn format_recent_activity_output(
    channel_name: &str,
    requested_limit: usize,
    rows: &[RecentActivityRow],
    unprocessed_recent: usize,
) -> String {
    let top_dates = rows
        .iter()
        .map(|row| short_date(row.video.published_at))
        .collect::<Vec<_>>();
    let timeframe = match (top_dates.first(), top_dates.last()) {
        (Some(first), Some(last)) if first == last => first.clone(),
        (Some(first), Some(last)) => format!("{last} to {first}"),
        _ => "unknown dates".to_string(),
    };

    let mut lines = vec![format!(
        "Recent library activity for {channel_name} based on the latest {} processed video{} currently in the library ({timeframe}):",
        rows.len(),
        if rows.len() == 1 { "" } else { "s" }
    )];

    for (index, row) in rows.iter().enumerate() {
        lines.push(format!(
            "{}. {} - {}",
            index + 1,
            short_date(row.video.published_at),
            row.video.title
        ));
        lines.push(format!(
            "   {}: {}",
            row.material.source_kind.as_str(),
            compact_excerpt(&row.material.content)
        ));
    }

    let mut notes = Vec::new();
    if rows.len() < 3 {
        notes.push(format!(
            "Only {} processed recent video{} were available, so the evidence is limited.",
            rows.len(),
            if rows.len() == 1 { "" } else { "s" }
        ));
    }
    if unprocessed_recent > 0 {
        notes.push(format!(
            "{unprocessed_recent} of the latest {requested_limit} videos are not fully processed yet."
        ));
    }
    notes.push(
        "This reflects recent videos in your library, not real-time off-platform activity."
            .to_string(),
    );

    lines.push("Notes:".to_string());
    for note in notes {
        lines.push(format!("- {note}"));
    }

    lines.join("\n")
}

fn compact_excerpt(input: &str) -> String {
    const MAX_CHARS: usize = 220;
    let compact = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_CHARS {
        compact
    } else {
        let mut clipped = compact.chars().take(MAX_CHARS).collect::<String>();
        clipped.push_str("...");
        clipped
    }
}

fn short_date(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn recent_markers() -> &'static [&'static str] {
    &[
        " recent ",
        " recently ",
        " lately ",
        " latest ",
        " these days ",
        " currently ",
        " right now ",
        " nowadays ",
    ]
}

fn normalize_text(input: &str) -> String {
    format!(
        " {} ",
        input
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch.is_whitespace() {
                    ch.to_ascii_lowercase()
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    )
}

#[cfg(test)]
mod tests {
    use super::{is_explicit_realtime_status_query, is_recent_activity_query};

    #[test]
    fn detects_recent_activity_queries() {
        assert!(is_recent_activity_query(
            "What is HealthyGamerGG doing lately?"
        ));
        assert!(is_recent_activity_query(
            "What has Theo been talking about recently?"
        ));
        assert!(is_recent_activity_query(
            "What is HealthyGamerGG focused on these days?"
        ));
    }

    #[test]
    fn detects_explicit_realtime_status_queries() {
        assert!(is_explicit_realtime_status_query(
            "Is HealthyGamerGG live right now?"
        ));
        assert!(is_explicit_realtime_status_query(
            "What is HealthyGamerGG working on this week outside YouTube?"
        ));
        assert!(!is_explicit_realtime_status_query(
            "What is HealthyGamerGG doing lately?"
        ));
    }
}
