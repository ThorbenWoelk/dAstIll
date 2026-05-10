use sha2::{Digest, Sha256};

use crate::services::text::limit_text as limit_text_base;

use super::{
    ChunkDraft, SEARCH_SUMMARY_MAX_CHUNKS, SEARCH_TRANSCRIPT_MAX_CHUNKS, SearchSourceKind,
};

const MAX_ERROR_DETAIL_CHARS: usize = 240;
const MAX_SNIPPET_CHARS: usize = 420;

pub fn hash_search_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn vector_to_json(embedding: &[f32]) -> String {
    let mut json = String::from("[");
    for (index, value) in embedding.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str(&format!("{value:.8}"));
    }
    json.push(']');
    json
}

fn required_target_words(total_words: usize, overlap_words: usize, max_chunks: usize) -> usize {
    if total_words == 0 {
        return 0;
    }

    let max_chunks = max_chunks.max(1);
    total_words
        .saturating_add(overlap_words.saturating_mul(max_chunks.saturating_sub(1)))
        .div_ceil(max_chunks)
}

pub fn build_embedding_input(
    video_title: &str,
    channel_name: &str,
    source_kind: SearchSourceKind,
    section_title: Option<&str>,
    chunk_text: &str,
) -> String {
    let mut input = format!(
        "Video: {video_title}\nChannel: {channel_name}\nSource: {}",
        source_kind.as_str()
    );
    if let Some(section_title) = section_title.filter(|title| !title.trim().is_empty()) {
        input.push_str(&format!("\nSection: {section_title}"));
    }
    input.push_str("\n\n");
    input.push_str(chunk_text.trim());
    input
}

fn normalize_source_text(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(strip_markdown_prefix)
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn strip_markdown_prefix(line: &str) -> &str {
    let trimmed = line.trim();
    trimmed
        .trim_start_matches('#')
        .trim_start_matches('-')
        .trim_start_matches('*')
        .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ')')
        .trim()
}

fn parse_markdown_sections(content: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_lines = Vec::<String>::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("## ") {
            if let Some(current_title) = current_title.take() {
                sections.push((current_title, current_lines.join("\n")));
                current_lines.clear();
            }
            current_title = Some(title.trim().to_string());
            continue;
        }

        if trimmed.starts_with("# ") && current_title.is_none() {
            continue;
        }

        if current_title.is_some() {
            current_lines.push(line.to_string());
        }
    }

    if let Some(current_title) = current_title.take() {
        sections.push((current_title, current_lines.join("\n")));
    }

    sections
}

fn push_normalized_paragraph(paragraphs: &mut Vec<String>, current_lines: &mut Vec<String>) {
    if current_lines.is_empty() {
        return;
    }

    let paragraph = normalize_source_text(&current_lines.join("\n"));
    current_lines.clear();
    if !paragraph.is_empty() {
        paragraphs.push(paragraph);
    }
}

fn split_paragraphs(content: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current_lines = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            push_normalized_paragraph(&mut paragraphs, &mut current_lines);
            continue;
        }
        current_lines.push(line.to_string());
    }
    push_normalized_paragraph(&mut paragraphs, &mut current_lines);

    paragraphs
}

fn split_words_into_chunks(text: &str, target_words: usize, overlap_words: usize) -> Vec<String> {
    let words = text.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < words.len() {
        let end = (start + target_words).min(words.len());
        chunks.push(words[start..end].join(" "));
        if end == words.len() {
            break;
        }
        let next_start = end.saturating_sub(overlap_words);
        if next_start <= start {
            start = end;
        } else {
            start = next_start;
        }
    }
    chunks
}

fn overlap_tail(text: &str, overlap_words: usize) -> String {
    let words = text.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() || overlap_words == 0 {
        return String::new();
    }
    let start = words.len().saturating_sub(overlap_words);
    words[start..].join(" ")
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

pub fn chunk_summary_content(content: &str, target_words: usize) -> Vec<ChunkDraft> {
    let normalized = normalize_source_text(content);
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut chunks = vec![ChunkDraft {
        source_kind: SearchSourceKind::Summary,
        section_title: None,
        text: normalized.clone(),
        word_count: count_words(&normalized),
        is_full_document: true,
        start_sec: None,
    }];

    let sections = parse_markdown_sections(content);
    if sections.is_empty() {
        return chunks;
    }

    let max_section_chunks = SEARCH_SUMMARY_MAX_CHUNKS.saturating_sub(1).max(1);
    let total_section_words: usize = sections
        .iter()
        .map(|(_, body)| count_words(&normalize_source_text(body)))
        .sum();
    let target_words = target_words.max(required_target_words(
        total_section_words,
        0,
        max_section_chunks,
    ));

    for (title, body) in sections {
        let normalized_body = normalize_source_text(&body);
        if normalized_body.is_empty() {
            continue;
        }

        if count_words(&normalized_body) <= target_words {
            chunks.push(ChunkDraft {
                source_kind: SearchSourceKind::Summary,
                section_title: Some(title),
                text: normalized_body,
                word_count: count_words(&body),
                is_full_document: false,
                start_sec: None,
            });
            continue;
        }

        for segment in split_words_into_chunks(&normalized_body, target_words, 0) {
            chunks.push(ChunkDraft {
                source_kind: SearchSourceKind::Summary,
                section_title: Some(title.clone()),
                word_count: count_words(&segment),
                text: segment,
                is_full_document: false,
                start_sec: None,
            });
        }
    }

    if chunks.len() > SEARCH_SUMMARY_MAX_CHUNKS {
        chunks.truncate(SEARCH_SUMMARY_MAX_CHUNKS);
    }

    chunks
}

/// Group timed caption segments into chunks by word-count target.
/// Each chunk's `start_sec` is the start of its first segment.
/// An overlap tail from the previous chunk is prepended (using that chunk's start_sec).
pub fn chunk_transcript_timed(
    segments: &[crate::models::TimedSegment],
    target_words: usize,
    overlap_words: usize,
) -> Vec<ChunkDraft> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut chunks: Vec<ChunkDraft> = Vec::new();
    let mut current_words: Vec<&str> = Vec::new();
    let mut current_start_sec: Option<f32> = None;
    let mut overlap_tail: Vec<String> = Vec::new();
    let mut overlap_start_sec: Option<f32> = None;

    for segment in segments {
        let seg_words: Vec<&str> = segment.text.split_whitespace().collect();
        if seg_words.is_empty() {
            continue;
        }

        // When adding this segment would exceed the target, flush first.
        if !current_words.is_empty() && current_words.len() + seg_words.len() > target_words {
            let text = if overlap_tail.is_empty() {
                current_words.join(" ")
            } else {
                format!("{} {}", overlap_tail.join(" "), current_words.join(" "))
            };
            let start = overlap_start_sec.or(current_start_sec);
            chunks.push(ChunkDraft {
                source_kind: SearchSourceKind::Transcript,
                section_title: None,
                word_count: count_words(&text),
                text,
                is_full_document: false,
                start_sec: start,
            });

            // Build overlap from end of current chunk.
            let all_words: Vec<String> = current_words.iter().map(|w| w.to_string()).collect();
            overlap_tail = if overlap_words > 0 {
                let start_idx = all_words.len().saturating_sub(overlap_words);
                all_words[start_idx..].to_vec()
            } else {
                Vec::new()
            };
            overlap_start_sec = current_start_sec;
            current_words.clear();
            current_start_sec = None;
        }

        if current_start_sec.is_none() {
            current_start_sec = Some(segment.start_sec);
        }
        current_words.extend(seg_words);
    }

    // Flush remaining words.
    if !current_words.is_empty() {
        let text = if overlap_tail.is_empty() {
            current_words.join(" ")
        } else {
            format!("{} {}", overlap_tail.join(" "), current_words.join(" "))
        };
        let start = overlap_start_sec.or(current_start_sec);
        chunks.push(ChunkDraft {
            source_kind: SearchSourceKind::Transcript,
            section_title: None,
            word_count: count_words(&text),
            text,
            is_full_document: false,
            start_sec: start,
        });
    }

    chunks
}

fn total_timed_segment_words(segments: &[crate::models::TimedSegment]) -> usize {
    segments
        .iter()
        .map(|segment| count_words(&segment.text))
        .sum()
}

fn group_paragraphs_into_chunks(
    paragraphs: &[String],
    target_words: usize,
    overlap_words: usize,
) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in paragraphs {
        let paragraph_words = count_words(paragraph);
        let current_words = count_words(&current);

        if !current.is_empty() && current_words + paragraph_words > target_words {
            let completed = current.trim().to_string();
            if !completed.is_empty() {
                chunks.push(completed.clone());
                current = overlap_tail(&completed, overlap_words);
                if !current.is_empty() {
                    current.push(' ');
                }
            } else {
                current.clear();
            }
        }

        if paragraph_words > target_words {
            for (index, split) in split_words_into_chunks(paragraph, target_words, overlap_words)
                .into_iter()
                .enumerate()
            {
                if index == 0 && current.is_empty() {
                    current = split;
                } else {
                    if !current.trim().is_empty() {
                        chunks.push(current.trim().to_string());
                    }
                    current = split;
                }
            }
        } else {
            current.push_str(paragraph);
        }

        if !current.is_empty() && !current.ends_with(' ') {
            current.push(' ');
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

pub fn chunk_transcript_content(
    content: &str,
    target_words: usize,
    overlap_words: usize,
    timed_segments: Option<&[crate::models::TimedSegment]>,
) -> Vec<ChunkDraft> {
    let total_words = timed_segments
        .filter(|segments| !segments.is_empty())
        .map(total_timed_segment_words)
        .unwrap_or_else(|| count_words(&normalize_source_text(content)));
    let target_words = target_words.max(required_target_words(
        total_words,
        overlap_words,
        SEARCH_TRANSCRIPT_MAX_CHUNKS,
    ));

    if let Some(segments) = timed_segments.filter(|s| !s.is_empty()) {
        return chunk_transcript_timed(segments, target_words, overlap_words);
    }

    let paragraphs = split_paragraphs(content);
    let chunks = if paragraphs.is_empty() {
        let normalized = normalize_source_text(content);
        if normalized.is_empty() {
            return Vec::new();
        }
        split_words_into_chunks(&normalized, target_words, overlap_words)
    } else {
        group_paragraphs_into_chunks(&paragraphs, target_words, overlap_words)
    };

    chunks
        .into_iter()
        .filter(|text| !text.is_empty())
        .map(|text| ChunkDraft {
            source_kind: SearchSourceKind::Transcript,
            section_title: None,
            word_count: count_words(&text),
            text,
            is_full_document: false,
            start_sec: None,
        })
        .collect()
}

fn limit_snippet(text: &str) -> String {
    let truncated = limit_text_base(text, MAX_SNIPPET_CHARS);
    if text.chars().count() > MAX_SNIPPET_CHARS {
        format!("{truncated}...")
    } else {
        truncated
    }
}

pub fn truncate_chunk_for_display(text: &str) -> String {
    limit_snippet(&normalize_source_text(text))
}

pub fn extract_keyword_snippet(text: &str, query_tokens: &[String]) -> String {
    let normalized = normalize_source_text(text);
    let total_chars = normalized.chars().count();

    if total_chars <= MAX_SNIPPET_CHARS {
        return normalized;
    }

    let lower = normalized.to_lowercase();
    let match_char_offset = query_tokens
        .iter()
        .filter_map(|token| {
            lower
                .find(token.as_str())
                .map(|byte_pos| lower[..byte_pos].chars().count())
        })
        .min();

    let Some(match_offset) = match_char_offset else {
        return limit_snippet(&normalized);
    };

    let all_chars: Vec<char> = normalized.chars().collect();
    let half_window = MAX_SNIPPET_CHARS / 2;
    let window_start = match_offset.saturating_sub(half_window);
    let window_end = (window_start + MAX_SNIPPET_CHARS).min(total_chars);
    let window_start = window_end.saturating_sub(MAX_SNIPPET_CHARS);

    let snippet: String = all_chars[window_start..window_end].iter().collect();
    let prefix = if window_start > 0 { "..." } else { "" };
    let suffix = if window_end < total_chars { "..." } else { "" };

    format!("{prefix}{}{suffix}", snippet.trim())
}

pub(super) fn limit_error_detail(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let truncated = limit_text_base(&collapsed, MAX_ERROR_DETAIL_CHARS);
    if collapsed.chars().count() > MAX_ERROR_DETAIL_CHARS {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
#[path = "content_processing_tests.rs"]
mod content_processing_tests;
