//! Scheduled background processing.
//!
//! The backend runs on Cloud Run with CPU only while a request is in flight,
//! and its workers pause while no user is active. Without readers, channels
//! are never refreshed and new videos are never summarized. Cloud Scheduler
//! calls this endpoint every few hours: it marks activity, refreshes every
//! channel, then keeps the request open while the queue works through
//! pending transcripts and summaries, up to a time limit.

use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Serialize;

use crate::{scheduler_identity::verify_scheduler_request, state::AppState, workers};

/// Longest a single run keeps the backend busy.
pub(crate) const CATCH_UP_WINDOW: Duration = Duration::from_secs(10 * 60);
/// How often the run checks whether work is left.
const PROGRESS_CHECK_INTERVAL: Duration = Duration::from_secs(15);
/// Upper bound when counting pending videos.
const PENDING_COUNT_LIMIT: usize = 1000;

static CATCH_UP_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize)]
pub struct CatchUpReport {
    pub pending_before: usize,
    pub pending_after: usize,
    pub elapsed_secs: u64,
}

/// Keep the run going while work remains and the window is open.
pub(crate) fn keep_catching_up(pending: usize, elapsed: Duration) -> bool {
    pending > 0 && elapsed < CATCH_UP_WINDOW
}

struct RunningGuard;

impl RunningGuard {
    fn acquire() -> Option<Self> {
        CATCH_UP_RUNNING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then_some(Self)
    }
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        CATCH_UP_RUNNING.store(false, Ordering::Release);
    }
}

pub async fn run_catch_up(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CatchUpReport>, (StatusCode, String)> {
    let Some(config) = state.catch_up.as_deref() else {
        return Err((StatusCode::NOT_FOUND, "Not found".to_string()));
    };
    if let Err(error) = verify_scheduler_request(headers.get(AUTHORIZATION), config).await {
        tracing::warn!(error = %error, "catch-up request rejected");
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }
    let Some(_running) = RunningGuard::acquire() else {
        return Err((StatusCode::CONFLICT, "Catch-up already running".to_string()));
    };

    let started = Instant::now();
    state.user_activity.touch();
    workers::refresh_all_channels(&state).await;

    let pending_before = count_pending(&state).await?;
    let mut pending = pending_before;
    tracing::info!(pending = pending_before, "catch-up started");
    while keep_catching_up(pending, started.elapsed()) {
        state.user_activity.touch();
        tokio::time::sleep(PROGRESS_CHECK_INTERVAL).await;
        pending = count_pending(&state).await?;
    }

    let report = CatchUpReport {
        pending_before,
        pending_after: pending,
        elapsed_secs: started.elapsed().as_secs(),
    };
    tracing::info!(
        pending_before = report.pending_before,
        pending_after = report.pending_after,
        elapsed_secs = report.elapsed_secs,
        "catch-up finished"
    );
    Ok(Json(report))
}

async fn count_pending(state: &AppState) -> Result<usize, (StatusCode, String)> {
    workers::count_pending_queue_work(state, PENDING_COUNT_LIMIT)
        .await
        .map_err(super::map_db_err)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
