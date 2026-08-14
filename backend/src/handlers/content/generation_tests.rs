use super::{
    MAX_SUMMARY_AUTO_REGEN_ATTEMPTS, completed_live_transcript_grace_elapsed,
    completed_live_transcript_looks_like_description, is_manual_summary_model,
    is_valid_cached_transcript, should_auto_regenerate_summary, summarizer_error_statuses,
    summarizer_pending_message, transcript_text,
};
use crate::models::{ContentStatus, Transcript, TranscriptRenderMode};
use crate::services::summarizer::SummarizerError;
use axum::http::StatusCode;

fn make_transcript(raw: Option<&str>, formatted: Option<&str>) -> Transcript {
    Transcript {
        video_id: "vid1".to_string(),
        raw_text: raw.map(ToOwned::to_owned),
        formatted_markdown: formatted.map(ToOwned::to_owned),
        render_mode: TranscriptRenderMode::PlainText,
        timed_text: None,
    }
}

#[test]
fn summarizer_temporary_errors_keep_summary_pending() {
    assert_eq!(
        summarizer_error_statuses(&SummarizerError::GenerationFailed(
            "subscription limit reached".to_string()
        )),
        (StatusCode::TOO_MANY_REQUESTS, ContentStatus::Pending)
    );
    assert_eq!(
        summarizer_error_statuses(&SummarizerError::NotAvailable),
        (StatusCode::SERVICE_UNAVAILABLE, ContentStatus::Pending)
    );
}

#[test]
fn summarizer_rate_limit_message_names_cloud_limit() {
    assert_eq!(
        summarizer_pending_message(&SummarizerError::GenerationFailed(
            "you have reached your weekly usage limit".to_string()
        )),
        "Ollama Cloud usage limit reached. The summary will retry when capacity returns."
    );
    assert_eq!(
        summarizer_pending_message(&SummarizerError::NotAvailable),
        "AI generation is temporarily unavailable. The summary will retry when capacity returns."
    );
}

#[test]
fn summarizer_non_temporary_errors_mark_summary_failed() {
    assert_eq!(
        summarizer_error_statuses(&SummarizerError::GenerationFailed(
            "malformed model output".to_string()
        )),
        (StatusCode::INTERNAL_SERVER_ERROR, ContentStatus::Failed)
    );
}

#[test]
fn valid_cached_transcript_accepts_real_content() {
    let t = make_transcript(Some("Hello world, this is a transcript."), None);
    assert!(is_valid_cached_transcript(&t));
}

#[test]
fn valid_cached_transcript_rejects_youtube_site_wide_blurb_in_raw_text() {
    let t = make_transcript(
        Some(
            "Enjoy the videos and music you love, upload original content, and share it all with friends, family, and the world on YouTube.\n",
        ),
        None,
    );
    assert!(!is_valid_cached_transcript(&t));
}

#[test]
fn valid_cached_transcript_rejects_youtube_site_wide_blurb_in_formatted_markdown() {
    let t = make_transcript(
        None,
        Some(
            "Enjoy the videos and music you love, upload original content, and share it all with friends, family, and the world on YouTube.\n",
        ),
    );
    assert!(!is_valid_cached_transcript(&t));
}

#[test]
fn valid_cached_transcript_rejects_empty_raw_text() {
    let t = make_transcript(Some("   "), None);
    assert!(!is_valid_cached_transcript(&t));
}

#[test]
fn valid_cached_transcript_rejects_all_none() {
    let t = make_transcript(None, None);
    assert!(!is_valid_cached_transcript(&t));
}

#[test]
fn valid_cached_transcript_falls_back_to_formatted_when_raw_is_none() {
    let t = make_transcript(None, Some("Actual transcript content here."));
    assert!(is_valid_cached_transcript(&t));
}

#[test]
fn completed_live_transcript_grace_waits_after_actual_end() {
    let ended = chrono::DateTime::parse_from_rfc3339("2026-04-18T22:27:42Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert!(!completed_live_transcript_grace_elapsed(
        ended,
        ended + chrono::Duration::minutes(2)
    ));
    assert!(completed_live_transcript_grace_elapsed(
        ended,
        ended + chrono::Duration::minutes(31)
    ));
}

#[test]
fn completed_live_transcript_rejects_description_like_text_for_long_stream() {
    let description = "Yesterday I tested Claude Design against v0, Grok, Google Stitch, Cursor, Droid, and ChatGPT Pro on the same blog redesign. Claude Design produced the visual system. Droid turned it into a working prototype with Opus on max thinking. It looked great locally. But a prototype that runs on your laptop is not a shipped product. Today I am closing the loop and getting my blog live on the internet.";
    let transcript = "Yesterday I tested Claude Design against v0, Grok, Google Stitch, Cursor, Droid, and ChatGPT Pro on the same blog redesign. Claude Design produced the visual system. Droid turned it into a working prototype with Opus on max thinking. It looked great locally. But a prototype that runs on your laptop is not a shipped product. Today I am closing the loop and getting my blog live on the internet.";

    assert!(completed_live_transcript_looks_like_description(
        transcript,
        description,
        Some(3 * 60 * 60),
        0
    ));
}

#[test]
fn completed_live_transcript_accepts_real_long_caption_text() {
    let description =
        "Today I am shipping a blog live on the internet after yesterday's prototype.";
    let transcript = (0..1_500)
        .map(|index| format!("caption{index}"))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(!completed_live_transcript_looks_like_description(
        &transcript,
        description,
        Some(3 * 60 * 60),
        0
    ));
}

#[test]
fn completed_live_transcript_accepts_timed_segments_even_when_short() {
    let description = "Yesterday I tested Claude Design against v0, Grok, Google Stitch, Cursor, Droid, and ChatGPT Pro on the same blog redesign. Claude Design produced the visual system. Droid turned it into a working prototype with Opus on max thinking. It looked great locally. But a prototype that runs on your laptop is not a shipped product. Today I am closing the loop and getting my blog live on the internet.";

    assert!(!completed_live_transcript_looks_like_description(
        description,
        description,
        Some(3 * 60 * 60),
        12
    ));
}

#[test]
fn is_manual_summary_model_matches_saved_edit_marker() {
    assert!(is_manual_summary_model(Some("manual")));
    assert!(is_manual_summary_model(Some("Manual")));
    assert!(!is_manual_summary_model(Some("glm-5.1:cloud")));
    assert!(!is_manual_summary_model(None));
}

#[test]
fn should_auto_regenerate_summary_requires_pending_or_loading_and_low_score() {
    assert!(should_auto_regenerate_summary(
        ContentStatus::Pending,
        Some(6),
        0,
        Some("glm-5.1:cloud")
    ));
    assert!(should_auto_regenerate_summary(
        ContentStatus::Loading,
        Some(0),
        1,
        None
    ));
    assert!(!should_auto_regenerate_summary(
        ContentStatus::Ready,
        Some(2),
        0,
        Some("glm-5.1:cloud")
    ));
    assert!(!should_auto_regenerate_summary(
        ContentStatus::Pending,
        Some(7),
        0,
        Some("glm-5.1:cloud")
    ));
    assert!(!should_auto_regenerate_summary(
        ContentStatus::Pending,
        None,
        0,
        Some("glm-5.1:cloud")
    ));
}

#[test]
fn should_auto_regenerate_summary_skips_user_saved_manual_summaries() {
    assert!(!should_auto_regenerate_summary(
        ContentStatus::Pending,
        Some(1),
        0,
        Some("manual")
    ));
    assert!(!should_auto_regenerate_summary(
        ContentStatus::Loading,
        Some(0),
        1,
        Some("Manual")
    ));
}

#[test]
fn should_auto_regenerate_summary_respects_max_attempts() {
    assert!(!should_auto_regenerate_summary(
        ContentStatus::Pending,
        Some(1),
        MAX_SUMMARY_AUTO_REGEN_ATTEMPTS,
        Some("glm-5.1:cloud")
    ));
}

#[test]
fn transcript_text_falls_back_to_formatted_markdown_when_raw_text_is_blank() {
    let transcript = Transcript {
        video_id: "video-123".to_string(),
        raw_text: Some("   ".to_string()),
        formatted_markdown: Some("## Section\nUseful formatted text".to_string()),
        render_mode: TranscriptRenderMode::Markdown,
        timed_text: None,
    };

    assert_eq!(
        transcript_text(&transcript),
        Some("## Section\nUseful formatted text")
    );
}

#[test]
fn transcript_text_prefers_non_empty_raw_text() {
    let transcript = Transcript {
        video_id: "video-123".to_string(),
        raw_text: Some("Raw transcript text".to_string()),
        formatted_markdown: Some("## Section\nFormatted text".to_string()),
        render_mode: TranscriptRenderMode::Markdown,
        timed_text: None,
    };

    assert_eq!(transcript_text(&transcript), Some("Raw transcript text"));
}
