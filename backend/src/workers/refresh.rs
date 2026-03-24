use std::time::Duration;

use tokio::time::sleep;
use tracing::Instrument;

use crate::{db, state::AppState};

use super::CHANNEL_REFRESH_INTERVAL;

/// Refresh all channels by fetching their RSS feeds and inserting new videos.
async fn refresh_all_channels(state: &AppState) {
    let span = logfire::span!("worker.refresh.batch");

    async move {
        let channels = {
            let conn = state.db.connect();
            db::list_channels(&conn)
                .await
                .map_err(|err| err.to_string())
        };

        let channels = match channels {
            Ok(list) => list,
            Err(err) => {
                tracing::error!(error = %err, "refresh worker failed to list channels");
                return;
            }
        };

        if channels.is_empty() {
            return;
        }

        tracing::info!(channel_count = channels.len(), "refreshing all channels");

        for (i, channel) in channels.iter().enumerate() {
            if i > 0 {
                sleep(Duration::from_secs(1)).await;
            }

            let channel_span =
                logfire::span!("worker.refresh.channel", channel.id = channel.id.clone(),);

            async {
                match state.youtube.fetch_videos(&channel.id).await {
                    Ok(videos) => {
                        let conn = state.db.connect();
                        let n = db::bulk_insert_videos(&conn, videos).await.unwrap_or(0);
                        if n > 0 {
                            state.read_cache.evict_channel(&channel.id).await;
                            tracing::info!(
                                channel_id = %channel.id,
                                new_videos = n,
                                "refresh worker found new videos"
                            );
                        }
                    }
                    Err(err) => {
                        tracing::warn!(
                            channel_id = %channel.id,
                            error = %err,
                            "refresh worker failed to fetch videos"
                        );
                    }
                }
            }
            .instrument(channel_span)
            .await;
        }
    }
    .instrument(span)
    .await;
}

pub fn spawn_refresh_worker(state: AppState) {
    let span = logfire::span!(
        "worker.refresh",
        interval_secs = CHANNEL_REFRESH_INTERVAL.as_secs(),
    );

    tokio::spawn(
        async move {
            tracing::info!(
                interval_secs = CHANNEL_REFRESH_INTERVAL.as_secs(),
                "channel refresh worker started"
            );

            // Run an initial refresh at startup so new videos appear immediately.
            refresh_all_channels(&state).await;

            loop {
                sleep(CHANNEL_REFRESH_INTERVAL).await;
                refresh_all_channels(&state).await;
            }
        }
        .instrument(span),
    );
}
