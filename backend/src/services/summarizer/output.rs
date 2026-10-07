//! Cleanup and checks for model-written summaries before they are stored.
//!
//! Models sometimes add text around the summary: reasoning ("Let me analyze this
//! transcript..."), `<think>` blocks, a draft followed by the final summary, a title line,
//! or closing chatter. [`clean_summary_output`] removes that text, and
//! [`check_summary_output`] decides whether what is left can be stored.

use thiserror::Error;

use super::transcript_size::is_han_or_kana;

/// The `##` sections the summary prompt asks for, in prompt order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SummarySection {
    /// `## At a glance`, or `## TL;DR` in older summaries.
    Glance,
    Overview,
    KeyPoints,
    Takeaways,
}

/// Why a cleaned summary cannot be stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SummaryOutputProblem {
    #[error("summary is empty after cleanup")]
    Empty,
    #[error("summary has no \"## At a glance\" section")]
    MissingGlanceSection,
    #[error("summary has no \"## Key Points\" section")]
    MissingKeyPoints,
    #[error("summary contains Chinese or Japanese characters that the transcript does not")]
    UnexpectedScript,
    #[error("summary looks cut off")]
    LooksTruncated,
    #[error("summary stopped at the model's output token limit")]
    HitOutputLimit,
}

const THINK_TAGS: &[(&str, &str)] = &[("<think>", "</think>"), ("<thinking>", "</thinking>")];

const CLOSING_CHATTER_PREFIXES: &[&str] = &[
    "let me know",
    "i hope this helps",
    "hope this helps",
    "i hope this summary",
    "hope this summary",
    "feel free to",
    "if you'd like",
    "if you would like",
    "would you like me",
    "do you want me",
    "is there anything else",
    "happy to help",
    "i can also",
];

/// Remove text the model wrote around the summary.
///
/// Steps, in order:
/// 1. Remove `<think>...</think>` blocks. When only a closing `</think>` is present and an
///    `## At a glance` heading follows it, everything before the tag is reasoning.
/// 2. Remove a code fence that wraps the whole output.
/// 3. Drop everything before the first expected section heading. When the full section
///    set appears more than once (a draft, then the final version), keep the last
///    complete set.
/// 4. Drop a leading title line such as `# Video Summary`.
/// 5. Drop closing chatter such as "Let me know if you want more detail."
pub(crate) fn clean_summary_output(raw: &str) -> String {
    let text = raw.replace("\r\n", "\n");
    let text = strip_reasoning_blocks(&text);
    let text = strip_wrapping_fence(&text);
    let lines: Vec<&str> = text.lines().collect();
    let lines = select_summary_sections(&lines);
    let lines = drop_leading_title_line(lines);
    let lines = drop_trailing_chatter(lines);
    lines.join("\n").trim().to_string()
}

/// Check a cleaned summary before it is stored as ready.
///
/// `source_text` is the transcript (and title) the summary was written from. It decides
/// whether Chinese or Japanese characters in the summary are expected.
pub(crate) fn check_summary_output(
    summary: &str,
    source_text: &str,
) -> Result<(), SummaryOutputProblem> {
    if summary.trim().is_empty() {
        return Err(SummaryOutputProblem::Empty);
    }

    let lines: Vec<&str> = summary.lines().collect();
    let sections: Vec<SummarySection> = section_headings(&lines)
        .into_iter()
        .map(|(_, section)| section)
        .collect();
    if !sections.contains(&SummarySection::Glance) {
        return Err(SummaryOutputProblem::MissingGlanceSection);
    }
    if !sections.contains(&SummarySection::KeyPoints) {
        return Err(SummaryOutputProblem::MissingKeyPoints);
    }
    if summary.chars().any(is_han_or_kana) && !source_text.chars().any(is_han_or_kana) {
        return Err(SummaryOutputProblem::UnexpectedScript);
    }
    if looks_cut_off(&lines) {
        return Err(SummaryOutputProblem::LooksTruncated);
    }
    Ok(())
}

fn section_heading(line: &str) -> Option<SummarySection> {
    let trimmed = line.trim();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=2).contains(&level) {
        return None;
    }
    let title = trimmed[level..]
        .trim()
        .trim_matches('*')
        .trim()
        .trim_end_matches(':')
        .trim()
        .to_ascii_lowercase();
    match title.as_str() {
        "at a glance" | "tl;dr" | "tldr" => Some(SummarySection::Glance),
        "overview" => Some(SummarySection::Overview),
        "key points" => Some(SummarySection::KeyPoints),
        "takeaways" => Some(SummarySection::Takeaways),
        _ => None,
    }
}

fn is_fence_line(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// Expected section headings with their line index, ignoring lines inside code fences.
fn section_headings(lines: &[&str]) -> Vec<(usize, SummarySection)> {
    let mut in_fence = false;
    let mut headings = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if is_fence_line(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(section) = section_heading(line) {
            headings.push((index, section));
        }
    }
    headings
}

/// Find a reasoning tag, ignoring case. A tag written as code (`` `<think>` ``) is part
/// of the summary text, not a real tag.
fn find_tag(haystack: &str, tag: &str, from: usize) -> Option<usize> {
    let lower = haystack.to_ascii_lowercase();
    lower[from..]
        .match_indices(tag)
        .map(|(at, _)| at + from)
        .find(|&at| !lower[..at].ends_with('`'))
}

fn rfind_tag(haystack: &str, tag: &str) -> Option<usize> {
    let lower = haystack.to_ascii_lowercase();
    lower
        .rmatch_indices(tag)
        .map(|(at, _)| at)
        .find(|&at| !lower[..at].ends_with('`'))
}

fn strip_reasoning_blocks(text: &str) -> String {
    let mut text = text.to_string();
    for (open, close) in THINK_TAGS {
        // Remove complete blocks. An opening tag without a closing tag is dropped on its
        // own; the section selection below removes the reasoning around it.
        while let Some(start) = find_tag(&text, open, 0) {
            match find_tag(&text, close, start + open.len()) {
                Some(close_start) => {
                    text.replace_range(start..close_start + close.len(), "\n");
                }
                None => text.replace_range(start..start + open.len(), ""),
            }
        }

        // A closing tag without an opening tag: some models omit `<think>` but still end
        // their reasoning with `</think>`. Others emit a stray `</think>` inside the summary.
        if let Some(last_close) = rfind_tag(&text, close) {
            let after = &text[last_close + close.len()..];
            let after_lines: Vec<&str> = after.lines().collect();
            let summary_follows = section_headings(&after_lines)
                .iter()
                .any(|(_, section)| *section == SummarySection::Glance);
            if summary_follows {
                text = format!("\n{after}");
            }
        }
        while let Some(start) = find_tag(&text, close, 0) {
            text.replace_range(start..start + close.len(), "\n");
        }
    }
    text
}

/// Remove a code fence around the summary: the first fence line opens it, the last
/// non-empty line closes it, and the summary's glance heading is inside. Text before the
/// opening fence is kept here and removed later with the rest of the preamble.
fn strip_wrapping_fence(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(open) = lines.iter().position(|line| is_fence_line(line)) else {
        return text.to_string();
    };
    let info = lines[open].trim().trim_start_matches('`').trim();
    if !matches!(info, "" | "markdown" | "md" | "text") {
        return text.to_string();
    }
    let Some(close) = lines.iter().rposition(|line| !line.trim().is_empty()) else {
        return text.to_string();
    };
    if close <= open || lines[close].trim() != "```" {
        return text.to_string();
    }
    let inner = &lines[open + 1..close];
    let summary_inside = inner
        .iter()
        .any(|line| section_heading(line) == Some(SummarySection::Glance));
    if !summary_inside {
        return text.to_string();
    }
    lines[..open]
        .iter()
        .chain(inner.iter())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
}

/// Keep the summary sections and drop text before them. When the section set repeats,
/// keep the last set that has both a glance section and Key Points.
fn select_summary_sections<'a, 'b>(lines: &'b [&'a str]) -> &'b [&'a str] {
    let headings = section_headings(lines);
    let Some(&(first_heading, _)) = headings.first() else {
        return lines;
    };

    // A new set starts when the glance or overview section repeats. Key Points can repeat
    // on purpose, for example in a summary of a show with two segments.
    let mut sets: Vec<Vec<(usize, SummarySection)>> = Vec::new();
    for heading in headings {
        let starts_new_set = matches!(heading.1, SummarySection::Glance | SummarySection::Overview)
            && sets
                .last()
                .is_none_or(|set| set.iter().any(|(_, section)| *section == heading.1));
        match sets.last_mut() {
            Some(set) if !starts_new_set => set.push(heading),
            _ => sets.push(vec![heading]),
        }
    }

    let is_complete = |set: &Vec<(usize, SummarySection)>| {
        set.iter()
            .any(|(_, section)| *section == SummarySection::Glance)
            && set
                .iter()
                .any(|(_, section)| *section == SummarySection::KeyPoints)
    };
    let Some(chosen) = sets.iter().rposition(is_complete) else {
        return &lines[first_heading..];
    };
    let start = sets[chosen][0].0;
    let end = sets
        .get(chosen + 1)
        .map(|next_set| next_set[0].0)
        .unwrap_or(lines.len());
    let selected = &lines[start..end];
    drop_unclosed_trailing_fence(selected)
}

/// A wrapping fence that started before the summary leaves a lone closing fence at the end.
fn drop_unclosed_trailing_fence<'a, 'b>(lines: &'b [&'a str]) -> &'b [&'a str] {
    let fence_count = lines.iter().filter(|line| is_fence_line(line)).count();
    if fence_count % 2 == 0 {
        return lines;
    }
    match lines.iter().rposition(|line| !line.trim().is_empty()) {
        Some(last) if lines[last].trim() == "```" => &lines[..last],
        _ => lines,
    }
}

fn drop_leading_title_line<'a, 'b>(lines: &'b [&'a str]) -> &'b [&'a str] {
    let Some(first) = lines.iter().position(|line| !line.trim().is_empty()) else {
        return lines;
    };
    let line = lines[first].trim();
    if section_heading(line).is_some() {
        return &lines[first..];
    }
    let is_title = line.starts_with("# ")
        || (line.starts_with('#') && line.to_ascii_lowercase().contains("summary"));
    if is_title {
        &lines[first + 1..]
    } else {
        &lines[first..]
    }
}

fn is_closing_chatter(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with('#') {
        return false;
    }
    let plain = trimmed
        .trim_start_matches(['*', '_', '>', ' '])
        .to_ascii_lowercase();
    CLOSING_CHATTER_PREFIXES
        .iter()
        .any(|prefix| plain.starts_with(prefix))
}

fn is_horizontal_rule(line: &str) -> bool {
    matches!(line.trim(), "---" | "***" | "___")
}

fn drop_trailing_chatter<'a, 'b>(lines: &'b [&'a str]) -> &'b [&'a str] {
    let mut end = lines.len();
    while end > 0 {
        let line = lines[end - 1];
        if line.trim().is_empty() || is_closing_chatter(line) || is_horizontal_rule(line) {
            end -= 1;
        } else {
            break;
        }
    }
    &lines[..end]
}

/// Signs that the model stopped mid-sentence: an open code fence, a heading with no body
/// at the end, or a last line that ends on a joining character or an open bold marker.
fn looks_cut_off(lines: &[&str]) -> bool {
    let fence_count = lines.iter().filter(|line| is_fence_line(line)).count();
    if fence_count % 2 == 1 {
        return true;
    }
    let Some(last) = lines
        .iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
    else {
        return true;
    };
    if last.starts_with('#') {
        return true;
    }
    if last.matches("**").count() % 2 == 1 {
        return true;
    }
    last.ends_with([',', ':', ';', '(', '[', '—', '–', '&'])
}

#[cfg(test)]
#[path = "output_tests.rs"]
mod tests;
