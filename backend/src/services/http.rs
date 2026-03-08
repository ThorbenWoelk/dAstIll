use reqwest::{Client, ClientBuilder};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CLOUD_COOLDOWN_DURATION: Duration = Duration::from_secs(3600);
const YOUTUBE_QUOTA_COOLDOWN_DURATION: Duration = Duration::from_secs(24 * 3600);
const TRANSCRIPT_COOLDOWN_DURATION: Duration = Duration::from_secs(60 * 60);

/// Shared cooldown state for cloud (`:cloud` suffix) models after HTTP 429.
/// Once activated, all cloud model attempts are skipped for 1 hour.
pub struct CloudCooldown {
    started_epoch_ms: AtomicU64,
}

impl Default for CloudCooldown {
    fn default() -> Self {
        Self::new()
    }
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

/// Shared cooldown state for YouTube Data API after quota exhaustion (403 quotaExceeded).
/// Once activated, all YouTube Data API attempts are skipped for 24 hours.
pub struct YouTubeQuotaCooldown {
    started_epoch_ms: AtomicU64,
}

impl Default for YouTubeQuotaCooldown {
    fn default() -> Self {
        Self::new()
    }
}

impl YouTubeQuotaCooldown {
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
        tracing::warn!(
            cooldown_hours = 24,
            "YouTube Data API quota cooldown activated - skipping Data API for 24 hours"
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
        elapsed_ms < YOUTUBE_QUOTA_COOLDOWN_DURATION.as_millis() as u64
    }
}

/// Shared cooldown state for YouTube transcript fetching after rate limits.
/// Once activated, all transcript extraction attempts are skipped for 1 hour.
pub struct TranscriptCooldown {
    started_epoch_ms: AtomicU64,
}

impl Default for TranscriptCooldown {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptCooldown {
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
        tracing::warn!(
            cooldown_hours = 1,
            "YouTube transcript cooldown activated - skipping transcript downloads for 1 hour"
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
        elapsed_ms < TRANSCRIPT_COOLDOWN_DURATION.as_millis() as u64
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

/// Helper to check if a model is "cloud".
/// Some providers expose names ending in `:cloud`, others in `-cloud`.
pub fn is_cloud_model(model: &str) -> bool {
    model.ends_with(":cloud") || model.ends_with("-cloud")
}

#[cfg(test)]
mod tests {
    use super::is_cloud_model;

    #[test]
    fn detects_cloud_models_with_colon_or_hyphen_suffixes() {
        assert!(is_cloud_model("minimax-m2.5:cloud"));
        assert!(is_cloud_model("qwen3-coder:480b-cloud"));
        assert!(!is_cloud_model("qwen3:8b"));
    }
}
