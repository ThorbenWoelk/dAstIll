use reqwest::Client;
use rig::completion::Prompt;
use rig::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Semaphore;
use tokio::time::{Instant as TokioInstant, timeout};

use crate::models::AiStatus;
use crate::services::http::CloudCooldown;
use crate::services::is_rate_limited;
use crate::services::ollama::{CooldownStatusPolicy, OllamaCore};

pub const MAX_TRANSCRIPT_FORMAT_ATTEMPTS: usize = 5;
pub const TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS: u64 = 300;
const TRANSCRIPT_FORMAT_HARD_TIMEOUT: Duration =
    Duration::from_secs(TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS);

const SUMMARY_PREAMBLE: &str = "You are a meticulous, comprehensive transcript-grounded summarizer. \
    Your summaries must capture ALL key points from the transcript - never skip or gloss over content. \
    Use only facts explicitly present in the provided transcript and title. Never invent details. \
    Scale summary length proportionally to transcript length.";
const TRANSCRIPT_CLEAN_PREAMBLE: &str = "You are a deterministic transcript formatter. Preserve transcript body tokens exactly and only improve layout.";

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

pub struct SummarizerService {
    core: OllamaCore,
}

impl SummarizerService {
    pub fn with_config(base_url: &str, model: &str) -> Self {
        Self {
            core: OllamaCore::new(base_url, model),
        }
    }

    pub fn with_client(client: Client, base_url: &str, model: &str) -> Self {
        Self {
            core: OllamaCore::with_client(client, base_url, model),
        }
    }

    pub fn with_fallback_model(mut self, fallback_model: Option<String>) -> Self {
        self.core = self.core.with_fallback_model(fallback_model);
        self
    }

    pub fn with_cloud_cooldown(mut self, cooldown: Arc<CloudCooldown>) -> Self {
        self.core = self.core.with_cloud_cooldown(cooldown);
        self
    }

    pub fn with_ollama_semaphore(mut self, semaphore: Arc<Semaphore>) -> Self {
        self.core = self.core.with_ollama_semaphore(semaphore);
        self
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
    ) -> Result<(String, String), SummarizerError> {
        let prompt = build_summary_prompt(transcript, video_title);

        let (raw, model_used) = self
            .prompt_model("summary", SUMMARY_PREAMBLE, &prompt)
            .await?;

        Ok((strip_summary_title_heading(&raw), model_used))
    }

    /// Clean transcript formatting while preserving token sequence.
    pub async fn clean_transcript_formatting(
        &self,
        transcript: &str,
    ) -> Result<TranscriptCleanResult, SummarizerError> {
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
            let (response, _model_used) = match timeout(
                remaining,
                self.prompt_model(&operation, TRANSCRIPT_CLEAN_PREAMBLE, &prompt),
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
                        "transcript clean compliance achieved after retry"
                    );
                }
                return Ok(TranscriptCleanResult {
                    content: response,
                    attempts_used: attempt,
                    max_attempts: MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                });
            }

            let mismatch = detect_transcript_mismatch(transcript, &response);
            tracing::warn!(
                attempt = attempt,
                max_attempts = MAX_TRANSCRIPT_FORMAT_ATTEMPTS,
                mismatch_index = mismatch.index,
                reason = mismatch.reason,
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

    pub fn model(&self) -> &str {
        self.core.model()
    }

    /// Returns `(response_text, model_used)`.
    async fn prompt_model(
        &self,
        operation: &str,
        preamble: &str,
        prompt: &str,
    ) -> Result<(String, String), SummarizerError> {
        tracing::info!(
            operation = operation,
            model = %self.model(),
            base_url = %self.core.base_url(),
            prompt_chars = prompt.len(),
            "starting ollama prompt"
        );
        let started = Instant::now();
        let ollama_client = self
            .core
            .build_ollama_client()
            .map_err(SummarizerError::GenerationFailed)?;

        // Skip cloud model entirely if cooldown is active
        let is_cloud = self.core.uses_cloud_model();
        let cooldown_active = self.core.is_cloud_cooldown_active();

        let (response, model_used) = if cooldown_active {
            let fallback = self.core.fallback_model().ok_or_else(|| {
                SummarizerError::GenerationFailed(
                    "cloud cooldown active and no fallback model configured".to_string(),
                )
            })?;
            tracing::info!(
                operation = operation,
                skipped_model = %self.model(),
                fallback_model = %fallback,
                "skipping cloud model due to active cooldown"
            );

            let _permit = self
                .core
                .acquire_local_permit(fallback)
                .await
                .map_err(SummarizerError::GenerationFailed)?;

            let fallback_agent = ollama_client.agent(fallback).preamble(preamble).build();
            let resp = fallback_agent.prompt(prompt).await?;
            (resp, fallback.to_string())
        } else {
            let _permit = self
                .core
                .acquire_local_permit(self.model())
                .await
                .map_err(SummarizerError::GenerationFailed)?;

            let agent = ollama_client.agent(self.model()).preamble(preamble).build();
            match agent.prompt(prompt).await {
                Ok(resp) => (resp, self.model().to_string()),
                Err(err) if is_rate_limited(&err) => {
                    if is_cloud {
                        self.core.activate_cloud_cooldown();
                    }
                    let fallback = self.core.fallback_model().ok_or_else(|| {
                        SummarizerError::GenerationFailed(format!(
                            "rate limited by provider and no fallback model configured: {err}"
                        ))
                    })?;
                    tracing::warn!(
                        operation = operation,
                        primary_model = %self.model(),
                        fallback_model = %fallback,
                        error = %err,
                        "rate limited - falling back to local model"
                    );

                    let _permit = self
                        .core
                        .acquire_local_permit(fallback)
                        .await
                        .map_err(SummarizerError::GenerationFailed)?;

                    let fallback_agent = ollama_client.agent(fallback).preamble(preamble).build();
                    let resp = fallback_agent.prompt(prompt).await?;
                    (resp, fallback.to_string())
                }
                Err(err) => return Err(err.into()),
            }
        };
        tracing::info!(
            operation = operation,
            model = %model_used,
            response_chars = response.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "completed ollama prompt"
        );
        if response.trim().is_empty() {
            return Err(SummarizerError::GenerationFailed(
                "Empty response from Ollama".to_string(),
            ));
        }
        Ok((response, model_used))
    }
}

pub(crate) fn transcript_text_equivalent(input: &str, output: &str) -> bool {
    let expected = input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let actual = normalized_output_tokens(output);
    expected == actual
}

fn build_clean_transcript_prompt(transcript: &str, retry_feedback: Option<&str>) -> String {
    let mut prompt = format!(
        r#"You must format transcript layout while preserving content exactly.

Transcript (source of truth):
<<<TRANSCRIPT_START>>>
{transcript}
<<<TRANSCRIPT_END>>>

Return markdown only.

Hard rules:
- Preserve transcript body tokens exactly - same words, same order, same punctuation.
- Do not add, remove, rewrite, summarize, or translate any transcript words.
- Allowed edits are layout-only: line breaks, paragraph breaks, and optional markdown section headings.
- If section headings are used, keep them concise and on separate lines.
- Never convert transcript body into lists or code blocks.
- Keep any <mark> wrappers inline and only around existing transcript phrases.

Safety fallback:
- If you are not fully certain about preserving tokens exactly, return the original transcript unchanged."#
    );

    if let Some(feedback) = retry_feedback {
        prompt.push_str("\n\nCompliance feedback from previous attempt:\n");
        prompt.push_str(feedback);
    }

    prompt
}

fn build_summary_prompt(transcript: &str, video_title: &str) -> String {
    let word_count = transcript.split_whitespace().count();
    let length_guidance = if word_count < 500 {
        "This is a short transcript. Keep the summary concise but still capture every point made."
    } else if word_count < 2000 {
        "This is a medium-length transcript. Provide a thorough summary that covers all topics discussed."
    } else if word_count < 5000 {
        "This is a long transcript. Provide a detailed, comprehensive summary. Use sub-sections under Key Points if the video covers multiple distinct topics. Every argument, example, and conclusion in the transcript should be reflected."
    } else {
        "This is a very long transcript. Provide an extensive, well-structured summary. Use sub-sections and group related points by theme. Leave nothing out - every significant topic, argument, example, data point, and conclusion must appear in the summary."
    };

    format!(
        r#"Video Title: {video_title}

Transcript (authoritative source - {word_count} words):
<<<TRANSCRIPT_START>>>
{transcript}
<<<TRANSCRIPT_END>>>

Length guidance: {length_guidance}

Task:
Create a comprehensive markdown summary grounded only in the transcript. The summary must capture ALL key points, arguments, examples, and conclusions from the transcript. Nothing should be skipped or glossed over.

Reliability rules:
- Use only information explicitly present in the transcript and title.
- Do not invent names, numbers, claims, timelines, or conclusions.
- If a point is uncertain or incomplete in the transcript, say so briefly.
- Keep wording precise and avoid speculative language.
- Start directly with section heading ## TL;DR - no top title line.

Output format (exact section headings):
## TL;DR
- Bullet-point list of the most important takeaways (3-7 bullets depending on content density).

## Overview
Factual overview paragraph covering the video's main subject, context, and purpose. Scale length with transcript length (2-3 sentences for short videos, a full paragraph for long ones).

## Key Points
Cover every distinct topic, argument, or segment from the transcript. Group related points under descriptive sub-headings if the video covers multiple themes. Each point must include the actual substance - not just a label but the specific claim, reasoning, or evidence from the transcript.

- **Point name**: transcript-grounded explanation with specifics (names, numbers, examples mentioned).
- **Point name**: transcript-grounded explanation with specifics.
(Add as many points as needed to fully represent the transcript content.)

## Takeaways
- Actionable or memorable takeaway grounded in transcript.
- Actionable or memorable takeaway grounded in transcript.
(Scale number of takeaways with content density.)"#
    )
}

#[derive(Debug, Clone)]
struct TranscriptMismatch {
    index: usize,
    reason: &'static str,
    expected_token: Option<String>,
    actual_token: Option<String>,
    expected_context: String,
    actual_context: String,
}

fn build_retry_feedback(mismatch: &TranscriptMismatch) -> String {
    format!(
        "Previous output failed transcript preservation.\n\
Reason: {reason}\n\
First mismatch index (0-based): {index}\n\
Expected token: {expected_token}\n\
Output token: {actual_token}\n\
Expected context: {expected_context}\n\
Output context: {actual_context}\n\
\n\
Fix this by preserving transcript body tokens exactly.\n\
Allowed transformations only:\n\
- section headings on separate lines\n\
- <mark> wrappers around existing phrases\n\
- whitespace and paragraph breaks\n\
Forbidden:\n\
- added, removed, reordered, or rewritten words",
        reason = mismatch.reason,
        index = mismatch.index,
        expected_token = mismatch.expected_token.as_deref().unwrap_or("<none>"),
        actual_token = mismatch.actual_token.as_deref().unwrap_or("<none>"),
        expected_context = mismatch.expected_context,
        actual_context = mismatch.actual_context
    )
}

fn detect_transcript_mismatch(input: &str, output: &str) -> TranscriptMismatch {
    let expected = input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let actual = normalized_output_tokens(output);
    detect_token_mismatch(&expected, &actual)
}

fn detect_token_mismatch(expected: &[String], actual: &[String]) -> TranscriptMismatch {
    let mut idx = 0usize;
    let min_len = expected.len().min(actual.len());
    while idx < min_len && expected[idx] == actual[idx] {
        idx += 1;
    }

    if idx < min_len {
        return TranscriptMismatch {
            index: idx,
            reason: "token mismatch",
            expected_token: Some(expected[idx].clone()),
            actual_token: Some(actual[idx].clone()),
            expected_context: token_window(expected, idx),
            actual_context: token_window(actual, idx),
        };
    }

    if expected.len() > actual.len() {
        return TranscriptMismatch {
            index: idx,
            reason: "output missing tokens",
            expected_token: expected.get(idx).cloned(),
            actual_token: None,
            expected_context: token_window(expected, idx),
            actual_context: token_window(actual, idx.saturating_sub(1)),
        };
    }

    TranscriptMismatch {
        index: idx,
        reason: "output has extra tokens",
        expected_token: None,
        actual_token: actual.get(idx).cloned(),
        expected_context: token_window(expected, idx.saturating_sub(1)),
        actual_context: token_window(actual, idx),
    }
}

fn token_window(tokens: &[String], center: usize) -> String {
    if tokens.is_empty() {
        return "<empty>".to_string();
    }
    let start = center.saturating_sub(4);
    let end = (center + 5).min(tokens.len());
    tokens[start..end].join(" ")
}

fn normalized_output_tokens(output: &str) -> Vec<String> {
    let body_only = output
        .lines()
        .filter_map(normalized_body_line)
        .collect::<Vec<_>>()
        .join("\n");
    let without_html = strip_html_tags(&body_only);
    let plain = strip_markdown_decorators(&without_html);
    let unescaped = unescape_markdown(&plain);
    unescaped
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn is_markdown_heading_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

fn normalized_body_line(line: &str) -> Option<String> {
    let mut trimmed = line.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    if is_markdown_heading_line(trimmed) || is_emphasis_heading_line(trimmed) {
        return None;
    }

    loop {
        let next = strip_known_prefix(trimmed);
        if next == trimmed {
            break;
        }
        trimmed = next;
    }

    if trimmed.is_empty() || is_markdown_heading_line(trimmed) || is_emphasis_heading_line(trimmed)
    {
        return None;
    }

    Some(trimmed.to_string())
}

fn is_emphasis_heading_line(line: &str) -> bool {
    let t = line.trim();
    (t.starts_with("**") && t.ends_with("**") && t.len() > 4)
        || (t.starts_with("__") && t.ends_with("__") && t.len() > 4)
}

fn strip_known_prefix(line: &str) -> &str {
    let t = line.trim_start();
    if let Some(rest) = t.strip_prefix("> ") {
        return rest;
    }
    if let Some(rest) = t.strip_prefix("- ") {
        return rest;
    }
    if let Some(rest) = t.strip_prefix("* ") {
        return rest;
    }
    if let Some(rest) = t.strip_prefix("+ ") {
        return rest;
    }
    if let Some(rest) = strip_ordered_list_prefix(t) {
        return rest;
    }
    t
}

fn strip_ordered_list_prefix(line: &str) -> Option<&str> {
    let mut chars = line.char_indices().peekable();
    let mut saw_digit = false;

    while let Some((_, ch)) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            saw_digit = true;
            let _ = chars.next();
        } else {
            break;
        }
    }

    if !saw_digit {
        return None;
    }

    let (_, sep) = chars.next()?;
    if sep != '.' && sep != ')' {
        return None;
    }

    let (space_idx, space) = chars.next()?;
    if !space.is_whitespace() {
        return None;
    }

    Some(line[space_idx + space.len_utf8()..].trim_start())
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn strip_markdown_decorators(input: &str) -> String {
    input
        .chars()
        .filter(|ch| !matches!(ch, '*' | '_' | '`'))
        .collect()
}

fn unescape_markdown(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.peek().copied() {
                if matches!(
                    next,
                    '\\' | '`'
                        | '*'
                        | '_'
                        | '{'
                        | '}'
                        | '['
                        | ']'
                        | '('
                        | ')'
                        | '#'
                        | '+'
                        | '-'
                        | '.'
                        | '!'
                        | '>'
                        | '|'
                ) {
                    out.push(next);
                    let _ = chars.next();
                    continue;
                }
            }
        }
        out.push(ch);
    }

    out
}

/// Strip a leading heading line that contains "summary" (case-insensitive).
/// LLMs tend to add titles like `# Summary: ...` or `## Video Summary: ...`
/// despite explicit prompt instructions not to.
fn strip_summary_title_heading(input: &str) -> String {
    let trimmed = input.trim_start();
    if let Some(rest) = trimmed.strip_prefix('#') {
        // Find the end of the heading line
        let heading_line = rest.split('\n').next().unwrap_or("");
        if heading_line.to_ascii_lowercase().contains("summary") {
            let after = &trimmed[1 + heading_line.len()..];
            return after.trim_start_matches('\n').to_string();
        }
    }
    input.to_string()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::timeout;

    use super::{
        MAX_TRANSCRIPT_FORMAT_ATTEMPTS, SummarizerService, build_clean_transcript_prompt,
        build_summary_prompt, detect_transcript_mismatch, strip_summary_title_heading,
        transcript_text_equivalent,
    };
    use crate::models::AiStatus;
    use crate::services::summary_evaluator::SummaryEvaluatorService;

    #[tokio::test]
    async fn is_available_returns_false_for_invalid_url() {
        let service = SummarizerService::with_config("://invalid-url", "qwen3:8b");
        assert!(!service.is_available().await);
    }

    #[tokio::test]
    async fn summarize_returns_error_for_invalid_url() {
        let service = SummarizerService::with_config("://invalid-url", "qwen3:8b");
        let result = service.summarize("test transcript", "test title").await;
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
    fn indicator_status_reports_cloud_when_primary_model_is_cloud_and_available() {
        let summarizer = SummarizerService::with_config("http://localhost:11434", "glm-5:cloud")
            .with_fallback_model(Some("qwen3-coder:30b".to_string()));

        assert_eq!(summarizer.indicator_status(false, true), AiStatus::Cloud);
    }

    #[test]
    fn indicator_status_reports_local_only_when_cloud_cooldown_uses_local_fallback() {
        let summarizer = SummarizerService::with_config("http://localhost:11434", "glm-5:cloud")
            .with_fallback_model(Some("qwen3-coder:30b".to_string()));

        assert_eq!(summarizer.indicator_status(true, true), AiStatus::LocalOnly);
    }

    #[test]
    fn indicator_status_reports_offline_when_cloud_cooldown_has_no_local_fallback() {
        let summarizer = SummarizerService::with_config("http://localhost:11434", "glm-5:cloud")
            .with_fallback_model(None);

        assert_eq!(summarizer.indicator_status(true, true), AiStatus::Offline);
    }

    #[test]
    fn indicator_status_reports_local_only_for_local_primary_model() {
        let summarizer =
            SummarizerService::with_config("http://localhost:11434", "qwen3-coder:30b");

        assert_eq!(
            summarizer.indicator_status(false, true),
            AiStatus::LocalOnly
        );
    }

    #[test]
    fn indicator_status_reports_offline_when_endpoint_is_unreachable() {
        let summarizer = SummarizerService::with_config("http://localhost:11434", "glm-5:cloud")
            .with_fallback_model(Some("qwen3-coder:30b".to_string()));

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
        let summarizer = SummarizerService::with_config(&ollama_url, &live_summary_model());
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
            summarizer.clean_transcript_formatting(transcript),
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
        let summarizer = SummarizerService::with_config(&ollama_url, &live_summary_model());
        let evaluator = SummaryEvaluatorService::with_config(&ollama_url, &live_evaluator_model());

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
            summarizer.summarize(transcript, title),
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
