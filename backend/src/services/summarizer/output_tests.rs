use super::{SummaryOutputProblem, check_summary_output, clean_summary_output};

const SUMMARY: &str = "## At a glance\n- The host compares two deployment styles.\n- Blue-green wins for risky launches.\n\n## Overview\nA short talk about rollout strategies.\n\n## Key Points\n- **Canary**: shifts traffic step by step.\n- **Blue-green**: flips all traffic at once.\n\n## Takeaways\n- Pick blue-green when rollback must be instant.";

const SOURCE: &str = "Deployment talk\nThe host compares canary and blue-green rollouts.";

#[test]
fn clean_summary_output_keeps_a_clean_summary_unchanged() {
    assert_eq!(clean_summary_output(SUMMARY), SUMMARY);
    assert_eq!(check_summary_output(SUMMARY, SOURCE), Ok(()));
}

#[test]
fn clean_summary_output_removes_think_blocks() {
    let raw = format!("<think>\nThe user wants a summary. Let me plan.\n</think>\n\n{SUMMARY}");
    assert_eq!(clean_summary_output(&raw), SUMMARY);

    let raw = format!("<THINKING>plan</THINKING>{SUMMARY}");
    assert_eq!(clean_summary_output(&raw), SUMMARY);
}

#[test]
fn clean_summary_output_drops_reasoning_ended_by_a_lone_closing_tag() {
    // Seen with glm-5.3: no opening tag, a draft set of headings, then `</think>` glued to
    // the first heading of the final summary.
    let raw = format!(
        "Let me analyze this transcript. It is short.\n\nOutput format:\n## At a glance\n- bullets\n\n## Overview\n2-3 sentences\n\n## Key Points\nCover each point.\n\n## Takeaways\n2-3 takeaways.\n\nLet me write it now.</think>{SUMMARY}"
    );
    assert_eq!(clean_summary_output(&raw), SUMMARY);
}

#[test]
fn clean_summary_output_keeps_content_around_a_stray_closing_tag_inside_the_summary() {
    let raw = SUMMARY.replace("at once.\n\n## Takeaways", "at once.</think>## Takeaways");
    let cleaned = clean_summary_output(&raw);
    assert!(cleaned.starts_with("## At a glance"));
    assert!(cleaned.contains("- **Blue-green**: flips all traffic at once.\n"));
    assert!(cleaned.contains("## Takeaways"));
    assert!(!cleaned.contains("think>"));
}

#[test]
fn clean_summary_output_drops_reasoning_before_the_first_heading() {
    let raw = format!(
        "The task is to summarize a short transcript (265 words).\n\n1. First idea.\n2. Second idea.\n\n{SUMMARY}"
    );
    assert_eq!(clean_summary_output(&raw), SUMMARY);
}

#[test]
fn clean_summary_output_keeps_the_last_complete_section_set() {
    let draft = "## At a glance\n- draft bullet\n\n## Overview\nDraft overview.\n\n## Key Points\n- draft point\n\n## Takeaways\n- draft takeaway\n\nNow the final version.";
    let raw = format!("{draft}\n\n{SUMMARY}");
    assert_eq!(clean_summary_output(&raw), SUMMARY);
}

#[test]
fn clean_summary_output_ignores_a_trailing_incomplete_restart() {
    let raw = format!("{SUMMARY}\n\n## At a glance\n- A second start that stops");
    assert_eq!(clean_summary_output(&raw), SUMMARY);
}

#[test]
fn clean_summary_output_keeps_a_repeated_key_points_section() {
    // A show with two segments can have two Key Points sections. Both stay.
    let two_segments = SUMMARY.replace(
        "\n\n## Takeaways",
        "\n\n## Key Points\n### Second segment\n- **Bonus**: a second topic.\n\n## Takeaways",
    );
    assert_eq!(clean_summary_output(&two_segments), two_segments);
}

#[test]
fn clean_summary_output_keeps_think_tags_written_as_code() {
    let summary = SUMMARY.replace(
        "shifts traffic step by step.",
        "the reward checks for `<think>` tags.",
    );
    assert_eq!(clean_summary_output(&summary), summary);
}

#[test]
fn clean_summary_output_keeps_sections_in_unusual_order_together() {
    let reordered = "## At a glance\n- One.\n\n## Key Points\n- **A**: b.\n\n## Overview\nText.\n\n## Takeaways\n- Done.";
    assert_eq!(clean_summary_output(reordered), reordered);
}

#[test]
fn clean_summary_output_accepts_legacy_tldr_heading() {
    let legacy = SUMMARY.replace("## At a glance", "## TL;DR");
    assert_eq!(clean_summary_output(&legacy), legacy);
    assert_eq!(check_summary_output(&legacy, SOURCE), Ok(()));
}

#[test]
fn clean_summary_output_strips_a_fence_around_the_whole_output() {
    let raw = format!("```markdown\n{SUMMARY}\n```");
    assert_eq!(clean_summary_output(&raw), SUMMARY);

    let raw = format!("Here is the summary:\n```\n{SUMMARY}\n```\n");
    assert_eq!(clean_summary_output(&raw), SUMMARY);
}

#[test]
fn clean_summary_output_keeps_code_blocks_inside_the_summary() {
    let with_code = SUMMARY.replace(
        "- **Canary**: shifts traffic step by step.",
        "- **Canary**: shifts traffic step by step.\n\n```yaml\n## Overview\nweight: 10\n```",
    );
    assert_eq!(clean_summary_output(&with_code), with_code);
    assert_eq!(check_summary_output(&with_code, SOURCE), Ok(()));
}

#[test]
fn clean_summary_output_drops_a_leading_title_line() {
    for title in [
        "# Deployment Strategy Tradeoffs",
        "# Summary: Deployment Strategy Tradeoffs",
        "## Video Summary: Deployment",
    ] {
        let raw = format!("{title}\n\n{SUMMARY}");
        assert_eq!(clean_summary_output(&raw), SUMMARY, "title: {title}");
    }
}

#[test]
fn clean_summary_output_drops_a_title_even_without_expected_sections() {
    let raw = "# Video Summary\n\nSome text without sections.";
    assert_eq!(clean_summary_output(raw), "Some text without sections.");
}

#[test]
fn clean_summary_output_drops_trailing_chatter() {
    let raw = format!(
        "{SUMMARY}\n\n---\n\nLet me know if you want a shorter version.\nI hope this helps!\n"
    );
    assert_eq!(clean_summary_output(&raw), SUMMARY);
}

#[test]
fn clean_summary_output_keeps_bullets_that_look_like_chatter() {
    let raw = format!("{SUMMARY}\n- Let me know in the comments, the host says.");
    assert_eq!(clean_summary_output(&raw), raw);
}

#[test]
fn clean_summary_output_returns_empty_for_reasoning_only_output() {
    let raw = "<think>I should summarize this, but I ran out of time.</think>";
    assert_eq!(clean_summary_output(raw), "");
    assert_eq!(
        check_summary_output(&clean_summary_output(raw), SOURCE),
        Err(SummaryOutputProblem::Empty)
    );
}

#[test]
fn check_summary_output_requires_glance_and_key_points() {
    let no_glance = SUMMARY.replace("## At a glance", "## Highlights");
    assert_eq!(
        check_summary_output(&no_glance, SOURCE),
        Err(SummaryOutputProblem::MissingGlanceSection)
    );

    let no_key_points = SUMMARY.replace("## Key Points", "## Details");
    assert_eq!(
        check_summary_output(&no_key_points, SOURCE),
        Err(SummaryOutputProblem::MissingKeyPoints)
    );

    // A heading glued into a sentence does not count as a section.
    let inline = format!(
        "Output format: ## At a glance, ## Key Points.\n{}",
        SUMMARY.replace("## At a glance\n", "")
    );
    assert_eq!(
        check_summary_output(&inline, SOURCE),
        Err(SummaryOutputProblem::MissingGlanceSection)
    );
}

#[test]
fn check_summary_output_rejects_han_or_kana_missing_from_the_source() {
    for leaked in ["avoid逃避ing", "Identity失效", "直观 view", "カタカナ"] {
        let summary = SUMMARY.replace("step by step", leaked);
        assert_eq!(
            check_summary_output(&summary, SOURCE),
            Err(SummaryOutputProblem::UnexpectedScript),
            "leaked: {leaked}"
        );
    }
}

#[test]
fn check_summary_output_allows_han_or_kana_present_in_the_source() {
    let summary = SUMMARY.replace("step by step", "as the speaker at 東京大学 explains");
    let source = "東京大学の講義です。The speaker compares rollouts.";
    assert_eq!(check_summary_output(&summary, source), Ok(()));
}

#[test]
fn check_summary_output_allows_other_non_latin_text() {
    let summary = SUMMARY.replace("step by step", "for the café in Zürich (Ελλάδα)");
    assert_eq!(check_summary_output(&summary, SOURCE), Ok(()));
}

#[test]
fn check_summary_output_flags_output_that_stops_mid_sentence() {
    let cut_after_comma = format!("{SUMMARY}\n- Also consider canary releases when,");
    assert_eq!(
        check_summary_output(&cut_after_comma, SOURCE),
        Err(SummaryOutputProblem::LooksTruncated)
    );

    let open_bold = format!("{SUMMARY}\n- **Rollback speed");
    assert_eq!(
        check_summary_output(&open_bold, SOURCE),
        Err(SummaryOutputProblem::LooksTruncated)
    );

    let empty_last_section = format!("{SUMMARY}\n\n### Extra notes");
    assert_eq!(
        check_summary_output(&empty_last_section, SOURCE),
        Err(SummaryOutputProblem::LooksTruncated)
    );

    let open_fence = format!("{SUMMARY}\n```bash\nkubectl rollout");
    assert_eq!(
        check_summary_output(&open_fence, SOURCE),
        Err(SummaryOutputProblem::LooksTruncated)
    );
}

#[test]
fn check_summary_output_accepts_bullets_without_final_punctuation() {
    let summary = format!("{SUMMARY}\n- Canary suits low-risk feature rollouts");
    assert_eq!(check_summary_output(&summary, SOURCE), Ok(()));
}
