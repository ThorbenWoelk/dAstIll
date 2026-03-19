use std::time::Duration;

use tokio::time::sleep;

const QUEUE_SCAN_LIMIT: usize = 4;
const QUEUE_POLL_INTERVAL: Duration = Duration::from_secs(5);
const QUEUE_IDLE_POLL_INTERVAL: Duration = Duration::from_secs(15);
const QUEUE_IDLE_POLL_MAX_INTERVAL: Duration = Duration::from_secs(60);
const CHANNEL_REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);
const CHANNEL_GAP_SCAN_INTERVAL: Duration = Duration::from_secs(10 * 60);
const CHANNEL_GAP_SCAN_LIMIT_PER_CHANNEL: usize = 8;
const SUMMARY_EVAL_SCAN_LIMIT: usize = 4;
const SUMMARY_EVAL_POLL_INTERVAL: Duration = Duration::from_secs(7);
const SUMMARY_EVAL_IDLE_POLL_INTERVAL: Duration = Duration::from_secs(30);
const SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL: Duration = Duration::from_secs(120);
const SEARCH_BACKFILL_SCAN_LIMIT: usize = 64;
const SEARCH_INDEX_SCAN_LIMIT: usize = 8;
const SEARCH_RECONCILE_SCAN_LIMIT: usize = 64;
const SEARCH_PRUNE_SCAN_LIMIT: usize = 256;
const SEARCH_INDEX_POLL_INTERVAL: Duration = Duration::from_secs(3);
const SEARCH_INDEX_IDLE_POLL_INTERVAL: Duration = Duration::from_secs(15);
const SEARCH_INDEX_IDLE_POLL_MAX_INTERVAL: Duration = Duration::from_secs(120);
const SEARCH_VECTOR_INDEX_BUILD_BACKLOG_THRESHOLD: usize = 128;
const SEARCH_RECONCILE_INTERVAL: Duration = Duration::from_secs(60);
const SEARCH_VECTOR_INDEX_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const MAX_DISTILLATION_RETRIES: u8 = 3;

mod gap_scan;
mod queue;
mod refresh;
mod search_index;
mod summary_evaluation;

pub use gap_scan::spawn_gap_scan_worker;
pub use queue::spawn_queue_worker;
pub use refresh::spawn_refresh_worker;
pub use search_index::spawn_search_index_worker;
pub use summary_evaluation::spawn_summary_evaluation_worker;

#[derive(Clone, Copy, Debug)]
struct PollBackoff {
    active_interval: Duration,
    idle_start_interval: Duration,
    idle_max_interval: Duration,
}

#[derive(Clone, Copy, Debug, Default)]
struct PollBackoffState {
    consecutive_idle_cycles: u32,
}

impl PollBackoff {
    const fn new(
        active_interval: Duration,
        idle_start_interval: Duration,
        idle_max_interval: Duration,
    ) -> Self {
        Self {
            active_interval,
            idle_start_interval,
            idle_max_interval,
        }
    }

    fn next_interval(&self, state: &mut PollBackoffState, had_activity: bool) -> Duration {
        if had_activity {
            state.consecutive_idle_cycles = 0;
            return self.active_interval;
        }

        let multiplier = 1u32
            .checked_shl(state.consecutive_idle_cycles.min(31))
            .unwrap_or(u32::MAX) as u128;
        state.consecutive_idle_cycles = state.consecutive_idle_cycles.saturating_add(1);

        let idle_millis = self.idle_start_interval.as_millis();
        let max_millis = self.idle_max_interval.as_millis();
        let next_millis = idle_millis.saturating_mul(multiplier).min(max_millis);
        let next_millis = next_millis.min(u64::MAX as u128) as u64;
        Duration::from_millis(next_millis)
    }
}

const QUEUE_POLL_BACKOFF: PollBackoff = PollBackoff::new(
    QUEUE_POLL_INTERVAL,
    QUEUE_IDLE_POLL_INTERVAL,
    QUEUE_IDLE_POLL_MAX_INTERVAL,
);
const SUMMARY_EVAL_POLL_BACKOFF: PollBackoff = PollBackoff::new(
    SUMMARY_EVAL_POLL_INTERVAL,
    SUMMARY_EVAL_IDLE_POLL_INTERVAL,
    SUMMARY_EVAL_IDLE_POLL_MAX_INTERVAL,
);
const SEARCH_INDEX_POLL_BACKOFF: PollBackoff = PollBackoff::new(
    SEARCH_INDEX_POLL_INTERVAL,
    SEARCH_INDEX_IDLE_POLL_INTERVAL,
    SEARCH_INDEX_IDLE_POLL_MAX_INTERVAL,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QueueTask {
    Transcript,
    Summary,
    Skip,
}

async fn sleep_with_backoff(
    backoff: PollBackoff,
    state: &mut PollBackoffState,
    had_activity: bool,
) {
    let delay = backoff.next_interval(state, had_activity);
    sleep(delay).await;
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::time::Duration;

    use super::queue::next_queue_task;
    use super::search_index::should_build_vector_index;
    use super::summary_evaluation::{
        should_queue_summary_auto_regeneration, should_run_summary_evaluation,
    };
    use super::{PollBackoff, PollBackoffState, QueueTask};
    use crate::db::SearchSourceCounts;
    use crate::models::{AiStatus, ContentStatus, Video};

    fn video_with_statuses(
        transcript_status: ContentStatus,
        summary_status: ContentStatus,
    ) -> Video {
        Video {
            id: "video".to_string(),
            channel_id: "channel".to_string(),
            title: "Title".to_string(),
            thumbnail_url: None,
            published_at: Utc::now(),
            is_short: false,
            transcript_status,
            summary_status,
            acknowledged: false,
            retry_count: 0,
            quality_score: None,
        }
    }

    #[test]
    fn next_queue_task_prioritizes_transcript_when_not_ready() {
        let video = video_with_statuses(ContentStatus::Pending, ContentStatus::Ready);
        assert_eq!(next_queue_task(&video), QueueTask::Transcript);

        let loading_video = video_with_statuses(ContentStatus::Loading, ContentStatus::Pending);
        assert_eq!(next_queue_task(&loading_video), QueueTask::Transcript);
    }

    #[test]
    fn next_queue_task_summarizes_only_after_transcript_ready() {
        let video = video_with_statuses(ContentStatus::Ready, ContentStatus::Pending);
        assert_eq!(next_queue_task(&video), QueueTask::Summary);

        let loading_summary = video_with_statuses(ContentStatus::Ready, ContentStatus::Loading);
        assert_eq!(next_queue_task(&loading_summary), QueueTask::Summary);
    }

    #[test]
    fn next_queue_task_retries_failed_rows() {
        let failed_transcript = video_with_statuses(ContentStatus::Failed, ContentStatus::Pending);
        assert_eq!(next_queue_task(&failed_transcript), QueueTask::Transcript);

        let failed_summary = video_with_statuses(ContentStatus::Ready, ContentStatus::Failed);
        assert_eq!(next_queue_task(&failed_summary), QueueTask::Summary);
    }

    #[test]
    fn next_queue_task_skips_complete_rows() {
        let done = video_with_statuses(ContentStatus::Ready, ContentStatus::Ready);
        assert_eq!(next_queue_task(&done), QueueTask::Skip);
    }

    #[test]
    fn should_queue_summary_auto_regeneration_only_for_low_scores_with_remaining_attempts() {
        assert!(should_queue_summary_auto_regeneration(6, 0));
        assert!(should_queue_summary_auto_regeneration(0, 1));
        assert!(!should_queue_summary_auto_regeneration(7, 0));
        assert!(!should_queue_summary_auto_regeneration(9, 0));
        assert!(!should_queue_summary_auto_regeneration(6, 2));
    }

    #[test]
    fn summary_evaluation_runs_only_when_it_wont_consume_local_fallback_capacity() {
        assert!(should_run_summary_evaluation(
            AiStatus::Cloud,
            "qwen3.5:397b-cloud"
        ));
        assert!(!should_run_summary_evaluation(
            AiStatus::LocalOnly,
            "qwen3.5:397b-cloud"
        ));
        assert!(should_run_summary_evaluation(
            AiStatus::LocalOnly,
            "qwen3:8b"
        ));
        assert!(!should_run_summary_evaluation(
            AiStatus::Offline,
            "qwen3.5:397b-cloud"
        ));
    }

    #[test]
    fn poll_backoff_uses_idle_start_then_doubles_until_max() {
        let backoff = PollBackoff::new(
            Duration::from_secs(3),
            Duration::from_secs(15),
            Duration::from_secs(60),
        );
        let mut state = PollBackoffState::default();

        assert_eq!(
            backoff.next_interval(&mut state, false),
            Duration::from_secs(15)
        );
        assert_eq!(
            backoff.next_interval(&mut state, false),
            Duration::from_secs(30)
        );
        assert_eq!(
            backoff.next_interval(&mut state, false),
            Duration::from_secs(60)
        );
        assert_eq!(
            backoff.next_interval(&mut state, false),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn poll_backoff_resets_to_active_interval_after_activity() {
        let backoff = PollBackoff::new(
            Duration::from_secs(5),
            Duration::from_secs(15),
            Duration::from_secs(60),
        );
        let mut state = PollBackoffState::default();

        assert_eq!(
            backoff.next_interval(&mut state, false),
            Duration::from_secs(15)
        );
        assert_eq!(
            backoff.next_interval(&mut state, false),
            Duration::from_secs(30)
        );
        assert_eq!(
            backoff.next_interval(&mut state, true),
            Duration::from_secs(5)
        );
        assert_eq!(
            backoff.next_interval(&mut state, false),
            Duration::from_secs(15)
        );
    }

    #[test]
    fn vector_index_build_waits_for_backlog_to_shrink_but_not_to_zero() {
        assert!(should_build_vector_index(&SearchSourceCounts {
            pending: 3,
            indexing: 115,
            ready: 6283,
            failed: 0,
            total_sources: 6401,
        }));

        assert!(!should_build_vector_index(&SearchSourceCounts {
            pending: 0,
            indexing: 129,
            ready: 6283,
            failed: 0,
            total_sources: 6412,
        }));

        assert!(!should_build_vector_index(&SearchSourceCounts {
            pending: 0,
            indexing: 0,
            ready: 0,
            failed: 0,
            total_sources: 0,
        }));
    }
}
