use std::time::Duration;

use tokio::time::timeout;

use super::prompts::{build_clean_transcript_prompt, build_summary_prompt};
use super::transcript_compare::{detect_transcript_mismatch, strip_summary_title_heading};
use super::{
    MAX_TRANSCRIPT_FORMAT_ATTEMPTS, SummarizerService, TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS,
    TRANSCRIPT_FORMAT_TIMEOUT_HEADROOM_SECS, apply_vocabulary_replacements,
    transcript_text_equivalent,
};
use crate::models::{AiStatus, VocabularyReplacement};
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
            &[],
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
    let formatted = "## Opening\nHello <mark>world.</mark>\n## Details\nThis is a test transcript.";
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
    let prompt = build_summary_prompt("alpha beta", "Sample Title", &[]);
    assert!(prompt.contains("<<<TRANSCRIPT_START>>>"));
    assert!(prompt.contains("<<<TRANSCRIPT_END>>>"));
    assert!(prompt.contains("Do not invent names, numbers, claims, timelines, or conclusions."));
    assert!(prompt.contains("Start directly with section heading ## At a glance"));
    assert!(prompt.contains("## Key Points"));
    assert!(prompt.contains("## Takeaways"));
    assert!(prompt.contains("## Overview"));
    assert!(prompt.contains("Length guidance:"));
    assert!(prompt.contains("Sponsor and ad segments:"));
}

#[test]
fn build_summary_prompt_scales_guidance_with_transcript_length() {
    let short = build_summary_prompt("word ".repeat(100).trim(), "Short", &[]);
    assert!(short.contains("short transcript"));

    let medium = build_summary_prompt(&"word ".repeat(1000), "Medium", &[]);
    assert!(medium.contains("medium-length transcript"));

    let long = build_summary_prompt(&"word ".repeat(3000), "Long", &[]);
    assert!(long.contains("long transcript"));

    let very_long = build_summary_prompt(&"word ".repeat(6000), "Very Long", &[]);
    assert!(very_long.contains("very long transcript"));
}

#[test]
fn build_summary_prompt_includes_vocabulary_guidance_when_rules_exist() {
    let replacements = vec![VocabularyReplacement {
        from: "Open A I".to_string(),
        to: "OpenAI".to_string(),
        added_at: chrono::Utc::now(),
    }];

    let prompt = build_summary_prompt("Open A I shipped a release.", "Sample", &replacements);

    assert!(prompt.contains("Preferred vocabulary replacements:"));
    assert!(prompt.contains("- `Open A I` -> `OpenAI`"));
}

#[test]
fn apply_vocabulary_replacements_applies_literal_rules_in_order() {
    let replacements = vec![
        VocabularyReplacement {
            from: "Open A I".to_string(),
            to: "OpenAI".to_string(),
            added_at: chrono::Utc::now(),
        },
        VocabularyReplacement {
            from: "San Franciso".to_string(),
            to: "San Francisco".to_string(),
            added_at: chrono::Utc::now(),
        },
    ];

    let result = apply_vocabulary_replacements("Open A I expanded in San Franciso.", &replacements);

    assert_eq!(result, "OpenAI expanded in San Francisco.");
}

#[test]
fn apply_vocabulary_replacements_skips_empty_and_identity_rules() {
    let replacements = vec![
        VocabularyReplacement {
            from: "".to_string(),
            to: "OpenAI".to_string(),
            added_at: chrono::Utc::now(),
        },
        VocabularyReplacement {
            from: "Anthropic".to_string(),
            to: "Anthropic".to_string(),
            added_at: chrono::Utc::now(),
        },
    ];

    let result = apply_vocabulary_replacements("Anthropic", &replacements);

    assert_eq!(result, "Anthropic");
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
        OllamaCore::new("http://localhost:11434", "glm-5.1:cloud")
            .with_fallback_model(Some("qwen3-coder:30b".to_string())),
    );

    assert_eq!(summarizer.indicator_status(false, true), AiStatus::Cloud);
}

#[test]
fn indicator_status_reports_local_only_when_cloud_cooldown_uses_local_fallback() {
    let summarizer = SummarizerService::new(
        OllamaCore::new("http://localhost:11434", "glm-5.1:cloud")
            .with_fallback_model(Some("qwen3-coder:30b".to_string())),
    );

    assert_eq!(summarizer.indicator_status(true, true), AiStatus::LocalOnly);
}

#[test]
fn indicator_status_reports_offline_when_cloud_cooldown_has_no_local_fallback() {
    let summarizer = SummarizerService::new(
        OllamaCore::new("http://localhost:11434", "glm-5.1:cloud").with_fallback_model(None),
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
        OllamaCore::new("http://localhost:11434", "glm-5.1:cloud")
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
    std::env::var("OLLAMA_SUMMARY_MODEL")
        .expect("OLLAMA_SUMMARY_MODEL must be set for live Ollama tests")
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
    let summarizer = SummarizerService::new(OllamaCore::new(&ollama_url, &live_summary_model()));
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
    let summarizer = SummarizerService::new(OllamaCore::new(&ollama_url, &live_summary_model()));
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
        summarizer.summarize(transcript, title, "test-video", "test-channel", &[]),
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

    let quality_score = evaluation
        .quality_score
        .expect("expected evaluator to return a numeric score");
    assert!(
        quality_score >= 7,
        "expected quality score >= 7, got {} ({:?})",
        quality_score,
        evaluation.quality_note
    );
}
