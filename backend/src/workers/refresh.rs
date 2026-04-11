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
                if let Ok(Some(profile)) = db::get_source_profile(&state.db, &channel.id).await {
                    if profile.source.provider != crate::models::ProviderKind::YouTube {
                        match crate::services::sync_source_profile(state, &profile).await {
                            Ok(n) if n > 0 => {
                                state.read_cache.evict_channel(&channel.id).await;
                                tracing::info!(
                                    channel_id = %channel.id,
                                    new_videos = n,
                                    "refresh worker synced non-youtube source"
                                );
                            }
                            Ok(_) => {}
                            Err(err) => {
                                tracing::warn!(
                                    channel_id = %channel.id,
                                    error = %err,
                                    "refresh worker failed to sync non-youtube source"
                                );
                            }
                        }
                        return;
                    }
                }

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
                if state.user_activity.is_idle() {
                    tracing::debug!("refresh worker skipped - no active user");
                    continue;
                }
                refresh_all_channels(&state).await;
            }
        }
        .instrument(span),
    );
}
