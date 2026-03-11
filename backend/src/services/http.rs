use reqwest::{Client, ClientBuilder};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CLOUD_COOLDOWN_DURATION: Duration = Duration::from_secs(3600);
const YOUTUBE_QUOTA_COOLDOWN_DURATION: Duration = Duration::from_secs(24 * 3600);
const TRANSCRIPT_COOLDOWN_DURATION: Duration = Duration::from_secs(60 * 60);

/// Generic cooldown timer backed by an atomic epoch-ms timestamp.
/// Once activated, `is_active()` returns true until `duration` elapses.
pub struct Cooldown {
    started_epoch_ms: AtomicU64,
    duration: Duration,
    label: &'static str,
}

impl Cooldown {
    fn new(duration: Duration, label: &'static str) -> Self {
        Self {
            started_epoch_ms: AtomicU64::new(0),
            duration,
            label,
        }
    }

    pub fn cloud() -> Self {
        Self::new(CLOUD_COOLDOWN_DURATION, "cloud model")
    }

    pub fn youtube_quota() -> Self {
        Self::new(YOUTUBE_QUOTA_COOLDOWN_DURATION, "YouTube Data API quota")
    }

    pub fn transcript() -> Self {
        Self::new(TRANSCRIPT_COOLDOWN_DURATION, "YouTube transcript")
    }

    pub fn activate(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.started_epoch_ms.store(now, Ordering::Relaxed);
        let remaining_min = self.duration.as_secs() / 60;
        tracing::warn!(
            cooldown_minutes = remaining_min,
            "{} cooldown activated for {} min",
            self.label,
            remaining_min,
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
        elapsed_ms < self.duration.as_millis() as u64
    }
}

pub type CloudCooldown = Cooldown;
pub type YouTubeQuotaCooldown = Cooldown;
pub type TranscriptCooldown = Cooldown;

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
        assert!(is_cloud_model("qwen3.5:397b-cloud"));
        assert!(!is_cloud_model("qwen3:8b"));
    }
}
