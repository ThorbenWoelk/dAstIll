mod prompts;
mod transcript_compare;

use std::time::Duration;
use thiserror::Error;
use tokio::time::{Instant as TokioInstant, timeout};
use tracing::Instrument;

use crate::models::AiStatus;
use crate::services::ollama::{
    CLOUD_PROMPT_TIMEOUT_SECS, CooldownStatusPolicy, OllamaCore, OllamaPromptError,
};

use prompts::{
    SUMMARY_PREAMBLE, TRANSCRIPT_CLEAN_PREAMBLE, build_clean_transcript_prompt,
    build_summary_prompt,
};
use transcript_compare::{
    build_retry_feedback, detect_transcript_mismatch, strip_summary_title_heading,
};

pub const MAX_TRANSCRIPT_FORMAT_ATTEMPTS: usize = 5;
pub const TRANSCRIPT_FORMAT_TIMEOUT_HEADROOM_SECS: u64 = 30;
pub const TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS: u64 =
    CLOUD_PROMPT_TIMEOUT_SECS - TRANSCRIPT_FORMAT_TIMEOUT_HEADROOM_SECS;
const TRANSCRIPT_FORMAT_HARD_TIMEOUT: Duration =
    Duration::from_secs(TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS);

#[derive(Debug, Clone)]
pub struct TranscriptCleanResult {
    pub content: String,
    pub attempts_used: usize,
    pub max_attempts: usize,
}

#[derive(Error, Debug)]
pub enum SummarizerError {
    #[error("Ollama request failed: {0}")]
    RequestFailed(#[from] rig::completion::PromptError),
    #[error("Ollama not available")]
    NotAvailable,
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    #[error(
        "Formatted transcript changed text content after {attempts_used}/{max_attempts} attempts"
    )]
    TextChanged {
        attempts_used: usize,
        max_attempts: usize,
    },
    #[error(
        "Transcript formatting timed out after {timeout_secs}s on attempt {attempts_used}/{max_attempts}"
    )]
    TimedOut {
        attempts_used: usize,
        max_attempts: usize,
        timeout_secs: u64,
    },
}

impl SummarizerError {
    pub fn is_rate_limited(&self) -> bool {
        let msg = self.to_string();
        msg.contains("rate limited") || msg.contains("429")
    }
}

pub struct SummarizerService {
    core: OllamaCore,
}

impl From<OllamaPromptError> for SummarizerError {
    fn from(err: OllamaPromptError) -> Self {
        match err {
            OllamaPromptError::NotAvailable => Self::NotAvailable,
            OllamaPromptError::RequestFailed(e) => Self::RequestFailed(e),
            OllamaPromptError::GenerationFailed(s) => Self::GenerationFailed(s),
            OllamaPromptError::EmptyResponse => {
                Self::GenerationFailed("Empty response from Ollama".to_string())
            }
        }
    }
}

impl SummarizerService {
    pub fn new(core: OllamaCore) -> Self {
        Self { core }
    }

    /// Check if Ollama is available.
    pub async fn is_available(&self) -> bool {
        self.core.is_available().await
    }

    pub fn indicator_status(
        &self,
        cloud_cooldown_active: bool,
        endpoint_available: bool,
    ) -> AiStatus {
        self.core.indicator_status(
            cloud_cooldown_active,
            endpoint_available,
            CooldownStatusPolicy::UseLocalFallback,
        )
    }

    /// Generate a summary from transcript text.
    /// Returns `(summary_content, model_used)`.
    pub async fn summarize(
        &self,
        transcript: &str,
        video_title: &str,
        video_id: &str,
        channel_id: &str,
    ) -> Result<(String, String), SummarizerError> {
        let span = logfire::span!(
            "summary.generate",
            video.id = video_id.to_string(),
            channel.id = channel_id.to_string(),
            transcript_chars = transcript.chars().count(),
            title_chars = video_title.chars().count(),
        );

        async move {
            let started = TokioInstant::now();
            let prompt = build_summary_prompt(transcript, video_title);

            let (raw, model_used) = self
                .prompt_model(
                    "summary",
                    SUMMARY_PREAMBLE,
                    &prompt,
                    Some(video_id),
                    Some(channel_id),
                )
                .await?;
            let summary = strip_summary_title_heading(&raw);

            tracing::info!(
                video_id = video_id,
                channel_id = channel_id,
                model = %model_used,
                summary_chars = summary.chars().count(),
                elapsed_ms = started.elapsed().as_millis() as u64,
                "summary generated"
            );

            Ok((summary, model_used))
        }
        .instrument(span)
        .await
    }

    /// Clean transcript formatting while preserving token sequence.
    pub async fn clean_transcript_formatting(
        &self,
        transcript: &str,
        video_id: &str,
        channel_id: &str,
    ) -> Result<TranscriptCleanResult, SummarizerError> {
        let span = logfire::span!(
            "transcript.clean",
            video.id = video_id.to_string(),
            channel.id = channel_id.to_string(),
            transcript_chars = transcript.chars().count(),
            max_attempts = MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
        );

        async move {
            let started = TokioInstant::now();
            let mut retry_feedback: Option<String> = None;

            for attempt in 1..=MAX_TRANSCRIPT_FORMAT_ATTEMPTS {
                let elapsed = started.elapsed();
                if elapsed >= TRANSCRIPT_FORMAT_HARD_TIMEOUT {
                    tracing::warn!(
                        attempts_used = attempt.saturating_sub(1),
                        max_attempts = MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                        hard_timeout_secs = TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS,
                        elapsed_ms = elapsed.as_millis() as u64,
                        "transcript clean hard timeout reached before new attempt"
                    );
                    return Err(SummarizerError::TimedOut {
                        attempts_used: attempt.saturating_sub(1),
                        max_attempts: MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                        timeout_secs: TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS,
                    });
                }

                let prompt = build_clean_transcript_prompt(transcript, retry_feedback.as_deref());
                let operation = format!("transcript_clean_attempt_{attempt}");
                let remaining = TRANSCRIPT_FORMAT_HARD_TIMEOUT.saturating_sub(elapsed);
                let (response, model_used) = match timeout(
                    remaining,
                    self.prompt_model(
                        &operation,
                        TRANSCRIPT_CLEAN_PREAMBLE,
                        &prompt,
                        Some(video_id),
                        Some(channel_id),
                    ),
                )
                .await
                {
                    Ok(result) => result?,
                    Err(_) => {
                        tracing::warn!(
                            attempts_used = attempt,
                            max_attempts = MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                            hard_timeout_secs = TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS,
                            elapsed_ms = started.elapsed().as_millis() as u64,
                            "transcript clean hard timeout reached during attempt"
                        );
                        return Err(SummarizerError::TimedOut {
                            attempts_used: attempt,
                            max_attempts: MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                            timeout_secs: TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS,
                        });
                    }
                };

                if transcript_text_equivalent(transcript, &response) {
                    if attempt > 1 {
                        tracing::info!(
                            attempt = attempt,
                            max_attempts = MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                            model = %model_used,
                            "transcript clean compliance achieved after retry"
                        );
                    }
                    let result = TranscriptCleanResult {
                        content: response,
                        attempts_used: attempt,
                        max_attempts: MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                    };
                    tracing::info!(
                        attempts_used = result.attempts_used,
                        max_attempts = result.max_attempts,
                        model = %model_used,
                        cleaned_chars = result.content.chars().count(),
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        "transcript clean completed"
                    );
                    return Ok(result);
                }

                let mismatch = detect_transcript_mismatch(transcript, &response);
                tracing::warn!(
                    attempt = attempt,
                    max_attempts = MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                    mismatch_index = mismatch.index,
                    reason = mismatch.reason,
                    model = %model_used,
                    "transcript clean compliance failed"
                );

                if attempt == MAX_TRANSCRIPT_FORMAT_ATTEMPTS {
                    return Err(SummarizerError::TextChanged {
                        attempts_used: attempt,
                        max_attempts: MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                    });
                }

                retry_feedback = Some(build_retry_feedback(&mismatch));
            }
            Err(SummarizerError::TextChanged {
                attempts_used: MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                max_attempts: MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
            })
        }
        .instrument(span)
        .await
    }

    pub fn model(&self) -> &str {
        self.core.model()
    }

    /// Returns `(response_text, model_used)`.
    async fn prompt_model(
        &self,
        operation: &str,
        preamble: &str,
        prompt: &str,
        video_id: Option<&str>,
        channel_id: Option<&str>,
    ) -> Result<(String, String), SummarizerError> {
        tracing::info!(
            operation = operation,
            video_id = video_id.unwrap_or("-"),
            channel_id = channel_id.unwrap_or("-"),
            "starting summarizer prompt"
        );
        self.core
            .prompt_with_fallback(
                operation,
                preamble,
                prompt,
                CooldownStatusPolicy::UseLocalFallback,
            )
            .await
            .map_err(Into::into)
    }
}

pub(crate) fn transcript_text_equivalent(input: &str, output: &str) -> bool {
    let expected = input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let actual = transcript_compare::normalized_output_tokens(output);
    expected == actual
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::timeout;

    use super::prompts::{build_clean_transcript_prompt, build_summary_prompt};
    use super::transcript_compare::{detect_transcript_mismatch, strip_summary_title_heading};
    use super::{
        MAX_TRANSCRIPT_FORMAT_ATTEMPTS, SummarizerService, TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS,
        TRANSCRIPT_FORMAT_TIMEOUT_HEADROOM_SECS, transcript_text_equivalent,
    };
    use crate::models::AiStatus;
    use crate::services::ollama::{CLOUD_PROMPT_TIMEOUT_SECS, OllamaCore};
    use crate::services::summary_evaluator::SummaryEvaluatorService;

    #[tokio::test]
    async fn is_available_returns_false_for_invalid_url() {
        let service = SummarizerService::new(OllamaCore::new("://invalid-url", "qwen3:8b"));
        assert!(!service.is_available().await);
    }

    #[tokio::test]
    async fn summarize_returns_error_for_invalid_url() {
        let service = SummarizerService::new(OllamaCore::new("://invalid-url", "qwen3:8b"));
        let result = service
            .summarize(
                "test transcript",
                "test title",
                "test-video",
                "test-channel",
            )
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn transcript_text_equivalent_ignores_whitespace_changes() {
        let original = "Hello world.\nThis is a test transcript.";
        let formatted = "Hello   world.\n\nThis is a test transcript.";
        assert!(transcript_text_equivalent(original, formatted));
    }

    #[test]
    fn transcript_text_equivalent_allows_headings_and_mark_highlights() {
        let original = "Hello world.\nThis is a test transcript.";
        let formatted =
            "## Opening\nHello <mark>world.</mark>\n## Details\nThis is a test transcript.";
        assert!(transcript_text_equivalent(original, formatted));
    }

    #[test]
    fn transcript_text_equivalent_allows_list_prefixes_and_emphasis_headings() {
        let original = "Hello world.\nThis is a test transcript.";
        let formatted = "**Opening**\n- Hello world.\n1. This is a test transcript.";
        assert!(transcript_text_equivalent(original, formatted));
    }

    #[test]
    fn transcript_text_equivalent_allows_markdown_escapes() {
        let original = "Use 3.14 now.";
        let formatted = "## Note\nUse 3\\.14 now\\.";
        assert!(transcript_text_equivalent(original, formatted));
    }

    #[test]
    fn transcript_text_equivalent_detects_word_changes() {
        let original = "Hello world.\nThis is a test transcript.";
        let formatted = "Hello world.\nThis is an edited transcript.";
        assert!(!transcript_text_equivalent(original, formatted));
    }

    #[test]
    fn detect_transcript_mismatch_reports_first_mismatch_context() {
        let original = "alpha beta gamma delta";
        let formatted = "## Title\nalpha beta zeta delta";
        let mismatch = detect_transcript_mismatch(original, formatted);
        assert_eq!(mismatch.index, 2);
        assert_eq!(mismatch.reason, "token mismatch");
        assert_eq!(mismatch.expected_token.as_deref(), Some("gamma"));
        assert_eq!(mismatch.actual_token.as_deref(), Some("zeta"));
    }

    #[test]
    fn strip_summary_title_heading_removes_hash_summary_colon() {
        let input = "# Summary: The 36-Month AI Crisis\n\n## Brief Overview\nContent";
        assert_eq!(
            strip_summary_title_heading(input),
            "## Brief Overview\nContent"
        );
    }

    #[test]
    fn strip_summary_title_heading_removes_video_summary() {
        let input = "## Video Summary: The Truth About High Performers\n\n### Overview";
        assert_eq!(strip_summary_title_heading(input), "### Overview");
    }

    #[test]
    fn strip_summary_title_heading_removes_trailing_summary() {
        let input = "# Cursor's Agents - Video Summary\n\n## Brief Overview";
        assert_eq!(strip_summary_title_heading(input), "## Brief Overview");
    }

    #[test]
    fn strip_summary_title_heading_preserves_non_summary_heading() {
        let input = "# Google AI Studio 2.0: Upgrade Overview\n\n## Brief Overview";
        assert_eq!(strip_summary_title_heading(input), input);
    }

    #[test]
    fn strip_summary_title_heading_preserves_body_with_summary_word() {
        let input = "## Overview\nThis is a summary of the video.";
        assert_eq!(strip_summary_title_heading(input), input);
    }

    #[test]
    fn build_summary_prompt_contains_strict_reliability_contract() {
        let prompt = build_summary_prompt("alpha beta", "Sample Title");
        assert!(prompt.contains("<<<TRANSCRIPT_START>>>"));
        assert!(prompt.contains("<<<TRANSCRIPT_END>>>"));
        assert!(
            prompt.contains("Do not invent names, numbers, claims, timelines, or conclusions.")
        );
        assert!(prompt.contains("Start directly with section heading ## TL;DR"));
        assert!(prompt.contains("## Key Points"));
        assert!(prompt.contains("## Takeaways"));
        assert!(prompt.contains("## Overview"));
        assert!(prompt.contains("Length guidance:"));
        assert!(prompt.contains("Sponsor and ad segments:"));
    }

    #[test]
    fn build_summary_prompt_scales_guidance_with_transcript_length() {
        let short = build_summary_prompt("word ".repeat(100).trim(), "Short");
        assert!(short.contains("short transcript"));

        let medium = build_summary_prompt(&"word ".repeat(1000), "Medium");
        assert!(medium.contains("medium-length transcript"));

        let long = build_summary_prompt(&"word ".repeat(3000), "Long");
        assert!(long.contains("long transcript"));

        let very_long = build_summary_prompt(&"word ".repeat(6000), "Very Long");
        assert!(very_long.contains("very long transcript"));
    }

    #[test]
    fn build_clean_transcript_prompt_contains_safety_fallback_and_feedback() {
        let prompt = build_clean_transcript_prompt("alpha beta gamma", Some("Mismatch at token 2"));
        assert!(prompt.contains("<<<TRANSCRIPT_START>>>"));
        assert!(prompt.contains("<<<TRANSCRIPT_END>>>"));
        assert!(prompt.contains("Safety fallback:"));
        assert!(prompt.contains("return the original transcript unchanged"));
        assert!(prompt.contains("Compliance feedback from previous attempt:"));
        assert!(prompt.contains("Mismatch at token 2"));
    }

    #[test]
    fn transcript_clean_timeout_leaves_response_headroom() {
        let hard_timeout_secs = std::hint::black_box(TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS);
        let timeout_headroom_secs = std::hint::black_box(TRANSCRIPT_FORMAT_TIMEOUT_HEADROOM_SECS);
        let cloud_prompt_timeout_secs = std::hint::black_box(CLOUD_PROMPT_TIMEOUT_SECS);

        assert_eq!(
            hard_timeout_secs + timeout_headroom_secs,
            cloud_prompt_timeout_secs
        );
        assert!(hard_timeout_secs < cloud_prompt_timeout_secs);
    }

    #[test]
    fn indicator_status_reports_cloud_when_primary_model_is_cloud_and_available() {
        let summarizer = SummarizerService::new(
            OllamaCore::new("http://localhost:11434", "glm-5:cloud")
                .with_fallback_model(Some("qwen3-coder:30b".to_string())),
        );

        assert_eq!(summarizer.indicator_status(false, true), AiStatus::Cloud);
    }

    #[test]
    fn indicator_status_reports_local_only_when_cloud_cooldown_uses_local_fallback() {
        let summarizer = SummarizerService::new(
            OllamaCore::new("http://localhost:11434", "glm-5:cloud")
                .with_fallback_model(Some("qwen3-coder:30b".to_string())),
        );

        assert_eq!(summarizer.indicator_status(true, true), AiStatus::LocalOnly);
    }

    #[test]
    fn indicator_status_reports_offline_when_cloud_cooldown_has_no_local_fallback() {
        let summarizer = SummarizerService::new(
            OllamaCore::new("http://localhost:11434", "glm-5:cloud").with_fallback_model(None),
        );

        assert_eq!(summarizer.indicator_status(true, true), AiStatus::Offline);
    }

    #[test]
    fn indicator_status_reports_local_only_for_local_primary_model() {
        let summarizer =
            SummarizerService::new(OllamaCore::new("http://localhost:11434", "qwen3-coder:30b"));

        assert_eq!(
            summarizer.indicator_status(false, true),
            AiStatus::LocalOnly
        );
    }

    #[test]
    fn indicator_status_reports_offline_when_endpoint_is_unreachable() {
        let summarizer = SummarizerService::new(
            OllamaCore::new("http://localhost:11434", "glm-5:cloud")
                .with_fallback_model(Some("qwen3-coder:30b".to_string())),
        );

        assert_eq!(summarizer.indicator_status(false, false), AiStatus::Offline);
    }

    fn live_ollama_tests_enabled() -> bool {
        std::env::var("RUN_LIVE_OLLAMA_TESTS")
            .map(|value| {
                let normalized = value.trim().to_ascii_lowercase();
                normalized == "1" || normalized == "true"
            })
            .unwrap_or(false)
    }

    fn live_ollama_url() -> String {
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string())
    }

    fn live_summary_model() -> String {
        std::env::var("OLLAMA_MODEL").expect("OLLAMA_MODEL must be set for live Ollama tests")
    }

    fn live_evaluator_model() -> String {
        std::env::var("SUMMARY_EVALUATOR_MODEL")
            .expect("SUMMARY_EVALUATOR_MODEL must be set for live Ollama tests")
    }

    #[tokio::test]
    #[ignore = "Live Ollama reliability test - run with RUN_LIVE_OLLAMA_TESTS=1 cargo test live_ollama -- --ignored --test-threads=1"]
    async fn live_ollama_transcript_clean_preserves_tokens() {
        if !live_ollama_tests_enabled() {
            return;
        }

        let ollama_url = live_ollama_url();
        let summarizer =
            SummarizerService::new(OllamaCore::new(&ollama_url, &live_summary_model()));
        assert!(
            summarizer.is_available().await,
            "Ollama is not reachable at {ollama_url}"
        );

        let transcript = "Host: Welcome back. Today we compare two rollout strategies for our API. \
Blue-green deployment keeps a full standby environment and flips traffic after health checks pass. \
Canary deployment shifts traffic gradually and watches error rates before continuing. \
For this team, the recommendation is blue-green because rollback must be instant during business hours.";

        let cleaned = timeout(
            Duration::from_secs(240),
            summarizer.clean_transcript_formatting(transcript, "test-video", "test-channel"),
        )
        .await
        .expect("transcript clean timed out")
        .expect("transcript clean call failed");

        assert!(
            transcript_text_equivalent(transcript, &cleaned.content),
            "cleaned transcript changed token sequence"
        );
        assert!(cleaned.attempts_used >= 1);
        assert!(cleaned.attempts_used <= MAX_TRANSCRIPT_FORMAT_ATTEMPTS);
        assert_eq!(cleaned.max_attempts, MAX_TRANSCRIPT_FORMAT_ATTEMPTS);
    }

    #[tokio::test]
    #[ignore = "Live Ollama reliability test - run with RUN_LIVE_OLLAMA_TESTS=1 cargo test live_ollama -- --ignored --test-threads=1"]
    async fn live_ollama_summary_has_required_sections_and_quality() {
        if !live_ollama_tests_enabled() {
            return;
        }

        let ollama_url = live_ollama_url();
        let summarizer =
            SummarizerService::new(OllamaCore::new(&ollama_url, &live_summary_model()));
        let evaluator =
            SummaryEvaluatorService::new(OllamaCore::new(&ollama_url, &live_evaluator_model()));

        assert!(
            summarizer.is_available().await,
            "Ollama is not reachable at {ollama_url}"
        );
        assert!(
            evaluator.is_available().await,
            "Ollama evaluator endpoint unavailable at {ollama_url}"
        );

        let title = "Deployment Strategy Tradeoffs";
        let transcript = "This episode compares canary and blue-green deployments. \
Canary releases move traffic in small increments and monitor metrics at each step. \
Blue-green keeps two full environments and switches all traffic once checks pass. \
The speaker says canary is cost-efficient for continuous experimentation, \
while blue-green is safer when instant rollback is required. \
Final recommendation: use blue-green for high-risk launches in peak business hours, \
and use canary for lower-risk feature rollouts.";

        let (summary, model_used) = timeout(
            Duration::from_secs(240),
            summarizer.summarize(transcript, title, "test-video", "test-channel"),
        )
        .await
        .expect("summary generation timed out")
        .expect("summary generation failed");

        assert!(!model_used.is_empty(), "model_used should not be empty");
        assert!(summary.contains("## Overview"), "missing Overview section");
        assert!(
            summary.contains("## Key Points"),
            "missing Key Points section"
        );
        assert!(
            summary.contains("## Takeaways"),
            "missing Takeaways section"
        );

        let evaluation = timeout(
            Duration::from_secs(240),
            evaluator.evaluate(transcript, &summary, title),
        )
        .await
        .expect("summary evaluation timed out")
        .expect("summary evaluation failed");

        assert!(
            evaluation.quality_score >= 7,
            "expected quality score >= 7, got {} ({:?})",
            evaluation.quality_score,
            evaluation.quality_note
        );
    }
}
