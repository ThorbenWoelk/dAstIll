use std::time::Duration;

use tokio::time::timeout;

use super::prompts::{build_clean_transcript_prompt, build_summary_prompt};
use super::transcript_compare::detect_transcript_mismatch;
use super::transcript_size::{
    LOCAL_CONTEXT_MAX_TOKENS, LOCAL_CONTEXT_MIN_TOKENS, SUMMARY_MAX_TIMEOUT_SECS,
    count_transcript_words, local_context_tokens, summary_timeout,
};
use super::{
    MAX_TRANSCRIPT_FORMAT_ATTEMPTS, SummarizerError, SummarizerService, SummaryOutputProblem,
    TRANSCRIPT_FORMAT_HARD_TIMEOUT_SECS, TRANSCRIPT_FORMAT_TIMEOUT_HEADROOM_SECS,
    apply_vocabulary_replacements, normalize_transcript_vocabulary, transcript_text_equivalent,
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

fn vocabulary_rule(from: &str, to: &str) -> VocabularyReplacement {
    VocabularyReplacement {
        from: from.to_string(),
        to: to.to_string(),
        added_at: chrono::Utc::now(),
    }
}

#[test]
fn normalize_transcript_vocabulary_matches_whole_words_only() {
    let rules = vec![vocabulary_rule("chat", "ChatGPT")];

    let result = normalize_transcript_vocabulary(
        "We were chatting about chat, chatbots, and the chat.",
        &rules,
    );

    assert_eq!(
        result,
        "We were chatting about ChatGPT, chatbots, and the ChatGPT."
    );
}

#[test]
fn normalize_transcript_vocabulary_skips_text_already_in_canonical_form() {
    let rules = vec![vocabulary_rule("Claude", "Claude Code")];

    let result = normalize_transcript_vocabulary("Claude Code is not Claude.", &rules);

    assert_eq!(result, "Claude Code is not Claude Code.");
}

#[test]
fn normalize_transcript_vocabulary_is_idempotent() {
    let rules = vec![
        vocabulary_rule("Claude", "Claude Code"),
        vocabulary_rule("code", "Code"),
        vocabulary_rule("Open A I", "OpenAI"),
    ];
    let transcript = "Claude wrote code for Open A I. Claude Code reviewed it.";

    let once = normalize_transcript_vocabulary(transcript, &rules);
    let twice = normalize_transcript_vocabulary(&once, &rules);

    assert_eq!(
        once,
        "Claude Code wrote Code for OpenAI. Claude Code reviewed it."
    );
    assert_eq!(twice, once);
}

#[test]
fn normalize_transcript_vocabulary_skips_canonical_form_that_ends_with_the_rule() {
    let rules = vec![vocabulary_rule("Pro", "Gemini Pro")];

    let result = normalize_transcript_vocabulary("Gemini Pro beats Pro. Product stays.", &rules);

    assert_eq!(result, "Gemini Pro beats Gemini Pro. Product stays.");
}

#[test]
fn normalize_transcript_vocabulary_handles_phrases_with_punctuation_edges() {
    let rules = vec![vocabulary_rule("C++", "C plus plus")];

    let result = normalize_transcript_vocabulary("I like C++ and C++20.", &rules);

    assert_eq!(result, "I like C plus plus and C plus plus20.");
}

#[test]
fn build_summary_prompt_states_language_and_output_rules() {
    let prompt = build_summary_prompt("alpha beta", "Sample Title", &[]);
    assert!(prompt.contains("Write the whole summary in English."));
    assert!(prompt.contains("Use the section headings exactly as written below."));
    assert!(prompt.contains("Chinese, Japanese, or any other non-Latin script"));
    assert!(prompt.contains("Do not include your reasoning"));
}

#[test]
fn build_summary_prompt_counts_unspaced_scripts_by_characters() {
    // 1,200 Han characters and no spaces: about 600 words, not one.
    let transcript = "東京".repeat(600);
    let prompt = build_summary_prompt(&transcript, "Sample", &[]);
    assert!(prompt.contains("600 words"));
    assert!(prompt.contains("medium-length transcript"));
}

#[test]
fn count_transcript_words_counts_spaced_text_by_tokens() {
    assert_eq!(count_transcript_words("one two  three\nfour"), 4);
    assert_eq!(count_transcript_words("well - okay"), 3);
    assert_eq!(count_transcript_words(""), 0);
}

#[test]
fn count_transcript_words_estimates_unspaced_scripts_from_characters() {
    // Ten Han and Kana characters count as five words.
    assert_eq!(count_transcript_words("東京タワーへ行きました"), 6);
    assert_eq!(count_transcript_words("東京タワーへ行きまし"), 5);
    // Thai has no spaces between words: about four characters per word.
    assert_eq!(count_transcript_words("สวัสดีครับ"), 3);
    // Mixed tokens count the Latin part as one word.
    assert_eq!(count_transcript_words("AI技術"), 2);
}

#[test]
fn summary_timeout_scales_with_transcript_length_up_to_a_cap() {
    assert_eq!(summary_timeout(0).as_secs(), CLOUD_PROMPT_TIMEOUT_SECS);
    assert_eq!(summary_timeout(5_000).as_secs(), CLOUD_PROMPT_TIMEOUT_SECS);
    assert_eq!(
        summary_timeout(15_000).as_secs(),
        CLOUD_PROMPT_TIMEOUT_SECS + 200
    );
    assert_eq!(
        summary_timeout(30_000).as_secs(),
        CLOUD_PROMPT_TIMEOUT_SECS + 500
    );
    assert_eq!(summary_timeout(200_000).as_secs(), SUMMARY_MAX_TIMEOUT_SECS);
}

#[test]
fn local_context_tokens_fits_long_prompts_within_bounds() {
    assert_eq!(
        local_context_tokens("preamble", "short"),
        LOCAL_CONTEXT_MIN_TOKENS
    );

    let long_prompt = "word ".repeat(20_000); // 100,000 chars
    let tokens = local_context_tokens("preamble", &long_prompt);
    assert!(
        tokens > 33_000,
        "expected room for the prompt, got {tokens}"
    );
    assert!(tokens <= LOCAL_CONTEXT_MAX_TOKENS);
    assert_eq!(tokens % 1_024, 0);

    let huge_prompt = "word ".repeat(200_000);
    assert_eq!(
        local_context_tokens("preamble", &huge_prompt),
        LOCAL_CONTEXT_MAX_TOKENS
    );

    // Han text uses about one token per character.
    let han_prompt = "東".repeat(20_000);
    assert!(local_context_tokens("", &han_prompt) >= 24_000);
}

/// Serve one canned `/api/generate` reply and return the base URL and the request body.
async fn serve_one_generate_reply(
    body: serde_json::Value,
) -> (String, tokio::task::JoinHandle<serde_json::Value>) {
    use axum::{Json, Router, routing::post};
    use std::sync::{Arc, Mutex};

    let seen = Arc::new(Mutex::new(serde_json::Value::Null));
    let seen_in_handler = seen.clone();
    let app = Router::new().route(
        "/api/generate",
        post(move |Json(request): Json<serde_json::Value>| {
            let seen = seen_in_handler.clone();
            let body = body.clone();
            async move {
                *seen.lock().unwrap() = request;
                Json(body)
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let handle = tokio::spawn(async move {
        // Give the caller time to send its request, then report what the server saw.
        for _ in 0..200 {
            if !seen.lock().unwrap().is_null() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        server.abort();
        seen.lock().unwrap().clone()
    });
    (format!("http://{address}"), handle)
}

const VALID_SUMMARY: &str = "## At a glance\n- Point one.\n\n## Overview\nOverview text.\n\n## Key Points\n- **Topic**: detail.\n\n## Takeaways\n- Takeaway.";

#[tokio::test]
async fn summarize_cleans_leaked_reasoning_before_returning() {
    let raw = format!(
        "Let me analyze this transcript first.\n\nPlan: cover the topic.</think>{VALID_SUMMARY}\n\nLet me know if you want more detail."
    );
    let (base_url, seen) = serve_one_generate_reply(serde_json::json!({
        "response": raw,
        "done": true,
        "done_reason": "stop"
    }))
    .await;
    let service = SummarizerService::new(OllamaCore::new(&base_url, "qwen3:8b"));

    let (summary, model) = service
        .summarize("A transcript about a topic.", "Title", "vid", "chan", &[])
        .await
        .expect("summary should be accepted after cleanup");

    assert_eq!(summary, VALID_SUMMARY);
    assert_eq!(model, "qwen3:8b");
    let request = seen.await.unwrap();
    assert_eq!(request["think"], serde_json::json!(false));
    assert!(
        request["options"]["num_ctx"].as_u64().unwrap() >= u64::from(LOCAL_CONTEXT_MIN_TOKENS),
        "local models get an explicit context window: {request}"
    );
}

#[tokio::test]
async fn summarize_rejects_output_that_hit_the_token_limit() {
    let (base_url, _seen) = serve_one_generate_reply(serde_json::json!({
        "response": VALID_SUMMARY,
        "done": true,
        "done_reason": "length"
    }))
    .await;
    let service = SummarizerService::new(OllamaCore::new(&base_url, "qwen3:8b"));

    let error = service
        .summarize("A transcript.", "Title", "vid", "chan", &[])
        .await
        .expect_err("length-limited output must not be stored");

    assert!(matches!(
        error,
        SummarizerError::RejectedOutput {
            problem: SummaryOutputProblem::HitOutputLimit,
            ..
        }
    ));
    assert!(!error.is_rate_limited());
}

#[tokio::test]
async fn summarize_rejects_output_without_summary_sections() {
    let (base_url, _seen) = serve_one_generate_reply(serde_json::json!({
        "response": "The transcript is short. I will think about it.",
        "done": true,
        "done_reason": "stop"
    }))
    .await;
    let service = SummarizerService::new(OllamaCore::new(&base_url, "qwen3:8b"));

    let error = service
        .summarize("A transcript.", "Title", "vid", "chan", &[])
        .await
        .expect_err("reasoning-only output must not be stored");

    assert!(matches!(
        error,
        SummarizerError::RejectedOutput {
            problem: SummaryOutputProblem::MissingGlanceSection,
            ..
        }
    ));
}

#[tokio::test]
async fn summarize_rejects_chinese_words_in_a_summary_of_an_english_transcript() {
    let leaked = VALID_SUMMARY.replace("detail.", "the plan停滞不前 after launch.");
    let (base_url, _seen) = serve_one_generate_reply(serde_json::json!({
        "response": leaked,
        "done": true,
        "done_reason": "stop"
    }))
    .await;
    let service = SummarizerService::new(OllamaCore::new(&base_url, "glm-5.1:cloud"));

    let error = service
        .summarize("An English transcript.", "Title", "vid", "chan", &[])
        .await
        .expect_err("mixed-script output must be retried");

    assert!(matches!(
        error,
        SummarizerError::RejectedOutput {
            problem: SummaryOutputProblem::UnexpectedScript,
            ..
        }
    ));
}

#[tokio::test]
async fn summarize_does_not_send_context_window_to_cloud_models() {
    let (base_url, seen) = serve_one_generate_reply(serde_json::json!({
        "response": VALID_SUMMARY,
        "done": true,
        "done_reason": "stop"
    }))
    .await;
    let service = SummarizerService::new(OllamaCore::new(&base_url, "glm-5.1:cloud"));

    service
        .summarize("A transcript.", "Title", "vid", "chan", &[])
        .await
        .expect("valid summary");

    let request = seen.await.unwrap();
    assert!(request["options"].get("num_ctx").is_none(), "{request}");
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
