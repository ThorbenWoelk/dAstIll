use crate::models::VocabularyReplacement;

pub(super) const SUMMARY_PREAMBLE: &str = "You are a meticulous, comprehensive transcript-grounded summarizer. \
    Your summaries must capture all substantive key points from the transcript; do not skip or gloss over editorial content. \
    If you are confident a portion is a paid promotion, sponsor read, or standalone ad segment, you may omit it from the summary. \
    Use only facts explicitly present in the provided transcript and title. Never invent details. \
    Scale summary length proportionally to transcript length.";
pub(super) const TRANSCRIPT_CLEAN_PREAMBLE: &str = "You are a deterministic transcript formatter. Preserve transcript body tokens exactly and only improve layout.";

pub(super) fn build_clean_transcript_prompt(
    transcript: &str,
    retry_feedback: Option<&str>,
) -> String {
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

pub(super) fn build_summary_prompt(
    transcript: &str,
    video_title: &str,
    vocabulary_replacements: &[VocabularyReplacement],
) -> String {
    let word_count = transcript.split_whitespace().count();
    let length_guidance = if word_count < 500 {
        "This is a short transcript. Keep the summary concise but still capture every point made."
    } else if word_count < 2000 {
        "This is a medium-length transcript. Provide a thorough summary that covers all topics discussed."
    } else if word_count < 5000 {
        "This is a long transcript. Provide a detailed, comprehensive summary. Use sub-sections under Key Points if the video covers multiple distinct topics. Every argument, example, and conclusion in the transcript should be reflected."
    } else {
        "This is a very long transcript. Provide an extensive, well-structured summary. Use sub-sections and group related points by theme. Cover every significant editorial topic, argument, example, data point, and conclusion (confidently identified sponsor or ad segments may be omitted as described in the task rules)."
    };

    let mut prompt = format!(
        r#"Video Title: {video_title}

Transcript (authoritative source - {word_count} words):
<<<TRANSCRIPT_START>>>
{transcript}
<<<TRANSCRIPT_END>>>

Length guidance: {length_guidance}

Task:
Create a comprehensive markdown summary grounded only in the transcript. The summary must capture all substantive key points, arguments, examples, and conclusions from the editorial content. Do not skip or gloss over meaningful parts of the main discussion.

Sponsor and ad segments:
- If you are confident that a contiguous stretch of the transcript is a paid promotion, sponsor read, discount pitch, or standalone ad break (clear verbal patterns, isolated product pitch, typical use-code or affiliate pitches, explicit sponsored-by style framing), you may leave it out of the summary entirely.
- When in doubt, include the material briefly rather than risk dropping real content.

Reliability rules:
- Use only information explicitly present in the transcript and title.
- Do not invent names, numbers, claims, timelines, or conclusions.
- If a point is uncertain or incomplete in the transcript, say so briefly.
- Keep wording precise and avoid speculative language.
- Start directly with section heading ## At a glance - no top title line.

Output format (exact section headings):
## At a glance
- Bullet-point list of the most important takeaways (3-7 bullets depending on content density).

## Overview
Factual overview paragraph covering the video's main subject, context, and purpose. Scale length with transcript length (2-3 sentences for short videos, a full paragraph for long ones).

## Key Points
Cover every distinct editorial topic, argument, or segment from the transcript (omit confidently identified sponsor or ad-only stretches). Group related points under descriptive sub-headings if the video covers multiple themes. Each point must include the actual substance - not just a label but the specific claim, reasoning, or evidence from the transcript.

- **Point name**: transcript-grounded explanation with specifics (names, numbers, examples mentioned).
- **Point name**: transcript-grounded explanation with specifics.
(Add as many points as needed to fully represent the transcript content.)

## Takeaways
- Actionable or memorable takeaway grounded in transcript.
- Actionable or memorable takeaway grounded in transcript.
(Scale number of takeaways with content density.)"#
    );

    if !vocabulary_replacements.is_empty() {
        prompt.push_str("\n\nPreferred vocabulary replacements:\n");
        prompt.push_str(
            "When the transcript contains one of these phrases, use the canonical spelling in the summary.\n",
        );
        for replacement in vocabulary_replacements {
            let from = replacement.from.trim();
            let to = replacement.to.trim();
            if from.is_empty() || to.is_empty() || from == to {
                continue;
            }
            prompt.push_str(&format!("- `{from}` -> `{to}`\n"));
        }
    }

    prompt
}
