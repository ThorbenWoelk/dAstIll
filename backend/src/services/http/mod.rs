use reqwest::{Client, ClientBuilder};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_CLOUD_COOLDOWN_DURATION: Duration = Duration::from_secs(5 * 24 * 3600);
const YOUTUBE_QUOTA_COOLDOWN_DURATION: Duration = Duration::from_secs(24 * 3600);
const TRANSCRIPT_COOLDOWN_DURATION: Duration = Duration::from_secs(60 * 60);
const DEFAULT_USER_IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

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
        Self::cloud_with_duration(DEFAULT_CLOUD_COOLDOWN_DURATION)
    }

    pub fn cloud_with_duration(duration: Duration) -> Self {
        Self::new(duration, "cloud model")
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

/// Tracks the last time a real user request was received.
/// Workers check `is_idle()` to skip cycles when nobody is using the app,
/// eliminating unnecessary reads during idle periods.
pub struct UserActivity {
    last_active_epoch_ms: AtomicU64,
    idle_timeout: Duration,
}

impl UserActivity {
    pub fn new(idle_timeout: Duration) -> Self {
        // Start as idle — workers wait until the first real request arrives.
        Self {
            last_active_epoch_ms: AtomicU64::new(0),
            idle_timeout,
        }
    }

    pub fn from_env() -> Self {
        let timeout = std::env::var("USER_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_USER_IDLE_TIMEOUT);
        Self::new(timeout)
    }

    /// Record that a user request just arrived.
    pub fn touch(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_active_epoch_ms.store(now, Ordering::Relaxed);
    }

    /// Returns `true` when no user request has been seen within the idle timeout.
    pub fn is_idle(&self) -> bool {
        let last = self.last_active_epoch_ms.load(Ordering::Relaxed);
        if last == 0 {
            return true;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let elapsed_ms = now.saturating_sub(last);
        elapsed_ms >= self.idle_timeout.as_millis() as u64
    }
}

pub fn build_http_client() -> Client {
    ClientBuilder::new()
        .user_agent("dastill/0.1")
        .timeout(Duration::from_secs(20))
        .build()
        .expect("http client build")
}

/// Detect provider capacity failures from HTTP status text and provider messages.
///
/// Ollama Cloud can return subscription-capacity failures as 403 responses.
/// Those should follow the same retry path as 429 provider throttling.
pub fn is_provider_capacity_limited_message(message: &str) -> bool {
    let msg = message.to_ascii_lowercase();
    (msg.contains("429") && msg.contains("too many requests"))
        || msg.contains("rate limited")
        || msg.contains("rate limit exceeded")
        || msg.contains("cloud cooldown active")
        || msg.contains("requires a subscription")
        || msg.contains("subscription limit")
        || msg.contains("quota exceeded")
        || msg.contains("usage limit")
}

/// Detect provider capacity errors from the rig completion error chain.
pub fn is_rate_limited(err: &rig::completion::PromptError) -> bool {
    is_provider_capacity_limited_message(&err.to_string())
}

/// Helper to check if a model is "cloud".
/// Some providers expose names ending in `:cloud`, others in `-cloud`.
pub fn is_cloud_model(model: &str) -> bool {
    model.ends_with(":cloud") || model.ends_with("-cloud")
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
