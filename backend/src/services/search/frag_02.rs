pub fn hash_search_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.trim().as_bytes());
    format!("{:x}", hasher.finalize())
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

    chunks
}

pub fn chunk_transcript_content(
    content: &str,
    target_words: usize,
    overlap_words: usize,
    timed_segments: Option<&[crate::models::TimedSegment]>,
) -> Vec<ChunkDraft> {
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

fn limit_snippet(text: &str) -> String {
    let truncated = limit_text_base(text, MAX_SNIPPET_CHARS);
    if text.chars().count() > MAX_SNIPPET_CHARS {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn limit_error_detail(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let truncated = limit_text_base(&collapsed, MAX_ERROR_DETAIL_CHARS);
    if collapsed.chars().count() > MAX_ERROR_DETAIL_CHARS {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
    use serde::Deserialize;
    use serde_json::json;

    use super::{
        SearchService, SearchSourceKind, build_embedding_input, chunk_summary_content,
        chunk_transcript_content, extract_keyword_snippet, fuse_ranked_matches,
        hash_search_content, truncate_chunk_for_display,
    };

    #[derive(Debug, Deserialize)]
    struct TestEmbedRequest {
        input: Vec<String>,
        dimensions: Option<usize>,
    }

    async fn spawn_embed_test_server(
        max_inputs_per_request: usize,
    ) -> (String, Arc<AtomicUsize>, tokio::sync::oneshot::Sender<()>) {
        let request_count = Arc::new(AtomicUsize::new(0));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind embed test server");
        let address = listener.local_addr().expect("embed test server address");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let app = Router::new().route(
            "/api/embed",
            post({
                let request_count = request_count.clone();
                move |Json(payload): Json<TestEmbedRequest>| {
                    let request_count = request_count.clone();
                    async move {
                        let request_number = request_count.fetch_add(1, Ordering::SeqCst) + 1;
                        if payload.input.len() > max_inputs_per_request {
                            return (
                                StatusCode::PAYLOAD_TOO_LARGE,
                                Json(json!({ "error": "too many inputs" })),
                            )
                                .into_response();
                        }

                        let dimensions = payload.dimensions.unwrap_or(2);
                        let embeddings = payload
                            .input
                            .iter()
                            .enumerate()
                            .map(|(index, _)| {
                                let mut embedding = vec![0.0; dimensions];
                                if dimensions > 0 {
                                    embedding[0] = request_number as f32;
                                }
                                if dimensions > 1 {
                                    embedding[1] = index as f32;
                                }
                                embedding
                            })
                            .collect::<Vec<_>>();

                        (StatusCode::OK, Json(json!({ "embeddings": embeddings }))).into_response()
                    }
                }
            }),
        );

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("run embed test server");
        });

        (format!("http://{address}"), request_count, shutdown_tx)
    }

    #[test]
    fn hash_search_content_changes_when_text_changes() {
        assert_ne!(
            hash_search_content("alpha beta"),
            hash_search_content("alpha gamma")
        );
    }

    #[test]
    fn chunk_summary_content_keeps_full_document_and_heading_sections() {
        let chunks = chunk_summary_content(
            "# Summary\n\n## Overview\nRust ownership basics and borrowing.\n\n## Tooling\nCargo workflows, tests, and release builds.",
            20,
        );

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].source_kind, SearchSourceKind::Summary);
        assert!(chunks[0].is_full_document);
        assert_eq!(chunks[1].section_title.as_deref(), Some("Overview"));
        assert_eq!(chunks[2].section_title.as_deref(), Some("Tooling"));
    }

    #[test]
    fn chunk_summary_content_keeps_full_summary_chunk_untruncated() {
        let long_body = std::iter::repeat_n("alpha beta gamma delta epsilon", 40)
            .collect::<Vec<_>>()
            .join(" ");
        let summary = format!("# Summary\n\n## Overview\n{long_body}");

        let chunks = chunk_summary_content(&summary, 20);

        assert!(chunks[0].is_full_document);
        assert_eq!(chunks[0].text, format!("Summary Overview {long_body}"));
        assert!(chunks[0].text.len() > super::MAX_SNIPPET_CHARS);
    }

    #[test]
    fn chunk_transcript_content_splits_long_paragraphs_with_overlap() {
        let transcript = [
            "Paragraph one introduces semantic search and vector indexes with practical examples.",
            "Paragraph two explains why keyword retrieval still matters for exact model names and acronyms.",
            "Paragraph three covers chunking tradeoffs and overlap decisions for transcript search.",
            "Paragraph four closes with deployment implications for local-only Ollama indexing.",
        ]
        .join("\n\n");

        let chunks = chunk_transcript_content(&transcript, 12, 4, None);

        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0].source_kind, SearchSourceKind::Transcript);
        assert!(!chunks[0].is_full_document);
        assert!(
            chunks
                .iter()
                .any(|chunk| chunk.text.contains("matters for exact model names"))
        );
    }

    #[test]
    fn chunk_transcript_content_respects_explicit_paragraph_breaks() {
        let transcript = "Alpha beta gamma delta.\n\nSecond paragraph starts here today.";

        let chunks = chunk_transcript_content(transcript, 5, 0, None);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].text, "Alpha beta gamma delta.");
        assert_eq!(chunks[1].text, "Second paragraph starts here today.");
    }

    #[test]
    fn extract_keyword_snippet_centers_the_matching_region_in_long_text() {
        let prefix = "alpha ".repeat(120);
        let suffix = "omega ".repeat(120);
        let text = format!("{prefix}semantic match appears here{suffix}");

        let snippet = extract_keyword_snippet(&text, &["semantic".to_string()]);

        assert!(snippet.contains("semantic match appears here"));
        assert!(snippet.starts_with("..."));
        assert!(snippet.ends_with("..."));
    }

    #[test]
    fn truncate_chunk_for_display_normalizes_markdown_noise() {
        let text = "# Heading\n\n- First point\n- Second point";

        assert_eq!(
            truncate_chunk_for_display(text),
            "Heading First point Second point"
        );
    }

    #[test]
    fn build_embedding_input_includes_search_metadata() {
        let input = build_embedding_input(
            "Video title",
            "Channel name",
            SearchSourceKind::Summary,
            Some("Overview"),
            "Key summary text",
        );

        assert!(input.contains("Video: Video title"));
        assert!(input.contains("Channel: Channel name"));
        assert!(input.contains("Source: summary"));
        assert!(input.contains("Section: Overview"));
        assert!(input.ends_with("Key summary text"));
    }

    #[test]
    fn fuse_ranked_matches_rewards_items_seen_by_both_retrievers() {
        let fused = fuse_ranked_matches(
            &[("chunk-a", 1), ("chunk-b", 2)],
            &[("chunk-b", 1), ("chunk-c", 2)],
            60.0,
        );

        assert_eq!(fused[0].0, "chunk-b");
        assert!(fused[0].1 > fused[1].1);
    }

    #[tokio::test]
    async fn embed_texts_splits_large_requests_into_multiple_batches() {
        let (base_url, request_count, shutdown_tx) = spawn_embed_test_server(8).await;
        let service = SearchService::with_config(&base_url, Some("embeddinggemma:latest"), 2, true);

        let inputs = (0..9)
            .map(|index| format!("chunk {index}"))
            .collect::<Vec<_>>();
        let embeddings = service
            .embed_texts(&inputs)
            .await
            .expect("batched embeddings");

        assert_eq!(request_count.load(Ordering::SeqCst), 2);
        assert_eq!(embeddings.len(), 9);
        assert_eq!(embeddings[0], vec![1.0, 0.0]);
        assert_eq!(embeddings[7], vec![1.0, 7.0]);
        assert_eq!(embeddings[8], vec![2.0, 0.0]);

        let _ = shutdown_tx.send(());
    }
}
