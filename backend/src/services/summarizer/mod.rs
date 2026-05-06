mod prompts;
mod transcript_compare;

use std::time::Duration;
use thiserror::Error;
use tokio::time::{Instant as TokioInstant, timeout};
use tracing::Instrument;

use crate::models::{AiStatus, VocabularyReplacement};
use crate::services::http::is_provider_capacity_limited_message;
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

fn normalize_vocabulary_entry(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub(crate) fn apply_vocabulary_replacements(
    transcript: &str,
    replacements: &[VocabularyReplacement],
) -> String {
    let mut normalized = transcript.to_string();

    for replacement in replacements {
        let Some(from) = normalize_vocabulary_entry(&replacement.from) else {
            continue;
        };
        let Some(to) = normalize_vocabulary_entry(&replacement.to) else {
            continue;
        };
        if from == to {
            continue;
        }
        normalized = normalized.replace(from, to);
    }

    normalized
}

pub(crate) fn transcript_text_equivalent(input: &str, output: &str) -> bool {
    let expected = input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let actual = transcript_compare::normalized_output_tokens(output);
    expected == actual
}

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
        is_provider_capacity_limited_message(&self.to_string())
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
            OllamaPromptError::InvalidStructuredResponse(s) => Self::GenerationFailed(s),
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

    pub fn defers_without_fallback_during_cloud_cooldown(&self) -> bool {
        self.core.defers_without_fallback_during_cloud_cooldown()
    }

    /// Generate a summary from transcript text.
    /// Returns `(summary_content, model_used)`.
    pub async fn summarize(
        &self,
        transcript: &str,
        video_title: &str,
        video_id: &str,
        channel_id: &str,
        vocabulary_replacements: &[VocabularyReplacement],
    ) -> Result<(String, String), SummarizerError> {
        let span = logfire::span!(
            "summary.generate",
            video.id = video_id.to_string(),
            channel.id = channel_id.to_string(),
            transcript_chars = transcript.chars().count(),
            title_chars = video_title.chars().count(),
            vocabulary_replacements = vocabulary_replacements.len(),
        );

        async move {
            let started = TokioInstant::now();
            let prompt = build_summary_prompt(transcript, video_title, vocabulary_replacements);

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

#[cfg(test)]
mod summarizer_tests;
