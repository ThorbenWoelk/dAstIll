#[derive(Debug, Clone)]
pub(crate) struct TranscriptMismatch {
    pub index: usize,
    pub reason: &'static str,
    pub expected_token: Option<String>,
    pub actual_token: Option<String>,
    pub expected_context: String,
    pub actual_context: String,
}

pub(super) fn build_retry_feedback(mismatch: &TranscriptMismatch) -> String {
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

pub(crate) fn detect_transcript_mismatch(input: &str, output: &str) -> TranscriptMismatch {
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

pub(super) fn normalized_output_tokens(output: &str) -> Vec<String> {
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
pub(super) fn strip_summary_title_heading(input: &str) -> String {
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
