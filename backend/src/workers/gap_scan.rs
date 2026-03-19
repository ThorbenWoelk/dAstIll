use std::collections::HashSet;
use std::time::Duration;

use tokio::time::sleep;

use crate::{db, state::AppState};

use super::{CHANNEL_GAP_SCAN_INTERVAL, CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL};

async fn fill_channel_gaps(
    state: &AppState,
    channel_id: &str,
    limit: usize,
    until: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<usize, String> {
    let known_video_ids = {
        let conn = state.db.connect();
        db::list_video_ids_by_channel(&conn, channel_id)
            .await
            .map_err(|err| err.to_string())?
            .into_iter()
            .collect::<HashSet<_>>()
    };

    let (videos, _exhausted) = state
        .youtube
        .fetch_videos_backfill_missing(channel_id, &known_video_ids, limit, until)
        .await
        .map_err(|err| err.to_string())?;

    let conn = state.db.connect();
    let inserted = db::bulk_insert_videos(&conn, videos)
        .await
        .map_err(|err| err.to_string())?;
    Ok(inserted)
}

async fn scan_all_channels_for_gaps(state: &AppState) {
    if state.youtube_quota_cooldown.is_active() {
        tracing::debug!("skipping gap scan worker - youtube quota cooldown active");
        return;
    }

    let channels = {
        let conn = state.db.connect();
        db::list_channels(&conn)
            .await
            .map_err(|err| err.to_string())
    };

    let channels = match channels {
        Ok(list) => list,
        Err(err) => {
            tracing::error!(error = %err, "gap scan worker failed to list channels");
            return;
        }
    };

    if channels.is_empty() {
        return;
    }

    tracing::info!(
        channel_count = channels.len(),
        per_channel_limit = CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
        "gap scan worker scanning channels"
    );

    for (i, channel) in channels.into_iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_secs(1)).await;
        }
        match fill_channel_gaps(
            state,
            &channel.id,
            CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
            channel.earliest_sync_date,
        )
        .await
        {
            Ok(inserted) if inserted > 0 => {
                tracing::info!(
                    channel_id = %channel.id,
                    inserted,
                    "gap scan worker inserted missing videos"
                );
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(
                    channel_id = %channel.id,
                    error = %err,
                    "gap scan worker failed for channel"
                );
            }
        }
    }
}

pub fn spawn_gap_scan_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!(
            interval_secs = CHANNEL_GAP_SCAN_INTERVAL.as_secs(),
            per_channel_limit = CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL,
            "channel gap scan worker started"
        );

        scan_all_channels_for_gaps(&state).await;

        loop {
            sleep(CHANNEL_GAP_SCAN_INTERVAL).await;
            scan_all_channels_for_gaps(&state).await;
        }
    });
}
