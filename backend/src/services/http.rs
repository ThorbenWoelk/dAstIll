use reqwest::{Client, ClientBuilder};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CLOUD_COOLDOWN_DURATION: Duration = Duration::from_secs(3600);

/// Shared cooldown state for cloud (`:cloud` suffix) models after HTTP 429.
/// Once activated, all cloud model attempts are skipped for 1 hour.
pub struct CloudCooldown {
    started_epoch_ms: AtomicU64,
}

impl CloudCooldown {
    pub fn new() -> Self {
        Self {
            started_epoch_ms: AtomicU64::new(0),
        }
    }

    pub fn activate(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.started_epoch_ms.store(now, Ordering::Relaxed);
        let remaining = CLOUD_COOLDOWN_DURATION.as_secs() / 60;
        tracing::warn!(
            cooldown_minutes = remaining,
            "cloud model cooldown activated - skipping cloud models for {remaining} min"
        );
    }

    pub fn is_active(&self) -> bool {
        let started = self.started_epoch_ms.load(Ordering::Relaxed);
        if started == 0 {
            return false;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let elapsed_ms = now.saturating_sub(started);
        elapsed_ms < CLOUD_COOLDOWN_DURATION.as_millis() as u64
    }
}

pub fn build_http_client() -> Client {
    ClientBuilder::new()
        .user_agent("dastill/0.1")
        .timeout(Duration::from_secs(20))
        .build()
        .expect("http client build")
}

/// Detect rate-limit (HTTP 429) errors from the rig completion error chain.
pub fn is_rate_limited(err: &rig::completion::PromptError) -> bool {
    let msg = err.to_string();
    msg.contains("429") && msg.contains("Too Many Requests")
}
