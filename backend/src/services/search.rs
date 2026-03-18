use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::services::http::build_http_client;

pub const SEARCH_EMBEDDING_DIMENSIONS: usize = 512;
pub const SEARCH_TRANSCRIPT_TARGET_WORDS: usize = 300;
pub const SEARCH_TRANSCRIPT_OVERLAP_WORDS: usize = 40;
pub const SEARCH_SUMMARY_TARGET_WORDS: usize = 300;
pub const SEARCH_RRF_K: f32 = 60.0;
const SEARCH_EMBED_BATCH_SIZE: usize = 8;
const SEARCH_EMBED_REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_ERROR_DETAIL_CHARS: usize = 240;
const MAX_SNIPPET_CHARS: usize = 420;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SearchSourceKind {
    Transcript,
    Summary,
}

impl SearchSourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transcript => "transcript",
            Self::Summary => "summary",
        }
    }

    pub fn from_db_value(value: &str) -> Self {
        match value {
            "summary" => Self::Summary,
            _ => Self::Transcript,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkDraft {
    pub source_kind: SearchSourceKind,
    pub section_title: Option<String>,
    pub text: String,
    pub word_count: usize,
    pub is_full_document: bool,
}

#[derive(Debug, Clone)]
pub struct SearchIndexChunk {
    pub chunk_index: usize,
    pub section_title: Option<String>,
    pub chunk_text: String,
    pub embedding_json: Option<String>,
    pub token_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchCandidate {
    pub chunk_id: String,
    pub video_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub video_title: String,
    pub source_kind: SearchSourceKind,
    pub section_title: Option<String>,
    pub chunk_text: String,
    pub published_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedSearchCandidate {
    pub candidate: SearchCandidate,
    pub score: f32,
}

#[derive(Debug)]
pub enum SearchError {
    Request(String),
    InvalidResponse(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(message) | Self::InvalidResponse(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for SearchError {}

pub struct SearchService {
    client: Client,
    base_url: String,
    model: Option<String>,
    api_key: Option<String>,
    dimensions: usize,
    semantic_enabled: bool,
    ollama_semaphore: Option<std::sync::Arc<Semaphore>>,
}

impl SearchService {
    pub fn with_config(
        base_url: &str,
        model: Option<&str>,
        dimensions: usize,
        semantic_enabled: bool,
    ) -> Self {
        Self {
            client: build_http_client(),
            base_url: base_url.to_string(),
            model: model.map(str::to_string),
            api_key: None,
            dimensions,
            semantic_enabled,
            ollama_semaphore: None,
        }
    }

    pub fn with_client(
        client: Client,
        base_url: &str,
        model: Option<&str>,
        dimensions: usize,
        semantic_enabled: bool,
    ) -> Self {
        Self {
            client,
            base_url: base_url.to_string(),
            model: model.map(str::to_string),
            api_key: None,
            dimensions,
            semantic_enabled,
            ollama_semaphore: None,
        }
    }

    pub fn with_ollama_semaphore(mut self, semaphore: std::sync::Arc<Semaphore>) -> Self {
        self.ollama_semaphore = Some(semaphore);
        self
    }

    pub fn with_api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key;
        self
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) => req.bearer_auth(key),
            None => req,
        }
    }

    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    pub fn model_label(&self) -> &str {
        self.model.as_deref().unwrap_or("disabled")
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    pub fn semantic_enabled(&self) -> bool {
        self.semantic_enabled
    }

    pub async fn is_available(&self) -> bool {
        if !self.semantic_enabled {
            return false;
        }
        let Some(model) = self.model.as_deref() else {
            return false;
        };
        let Ok(response) = self
            .auth(self.client.get(format!("{}/api/tags", self.base_url)))
            .send()
            .await
        else {
            return false;
        };
        if !response.status().is_success() {
            return false;
        }
        let Ok(payload) = response.json::<TagsResponse>().await else {
            return false;
        };
        let expected_latest = format!("{model}:latest");
        payload
            .models
            .into_iter()
            .any(|candidate| candidate.name == model || candidate.name == expected_latest)
    }

    pub async fn embed_texts(&self, input: &[String]) -> Result<Vec<Vec<f32>>, SearchError> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        if !self.semantic_enabled {
            return Err(SearchError::Request(
                "semantic search is disabled".to_string(),
            ));
        }
        let Some(model) = self.model.as_ref() else {
            return Err(SearchError::Request(
                "search embedding model is not configured".to_string(),
            ));
        };

        let _permit = self.acquire_local_permit().await?;
        let total_batches = input.len().div_ceil(SEARCH_EMBED_BATCH_SIZE);
        let mut embeddings = Vec::with_capacity(input.len());
        for (batch_index, batch) in input.chunks(SEARCH_EMBED_BATCH_SIZE).enumerate() {
            let batch_embeddings = self.embed_batch(model, batch).await.map_err(|err| {
                SearchError::Request(format!(
                    "embed batch {}/{} failed for {} chunks: {}",
                    batch_index + 1,
                    total_batches,
                    batch.len(),
                    err
                ))
            })?;
            embeddings.extend(batch_embeddings);
        }

        Ok(embeddings)
    }

    async fn embed_batch(
        &self,
        model: &str,
        input: &[String],
    ) -> Result<Vec<Vec<f32>>, SearchError> {
        let response = self
            .auth(
                self.client
                    .post(format!("{}/api/embed", self.base_url))
                    .timeout(SEARCH_EMBED_REQUEST_TIMEOUT)
                    .json(&EmbedRequest {
                        model: model.to_string(),
                        input,
                        dimensions: Some(self.dimensions),
                    }),
            )
            .send()
            .await
            .map_err(|err| SearchError::Request(err.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let detail = response
                .text()
                .await
                .ok()
                .map(|text| limit_error_detail(&text))
                .filter(|text| !text.is_empty());
            return Err(SearchError::Request(match detail {
                Some(detail) => format!("Ollama embed request failed ({status}): {detail}"),
                None => format!("Ollama embed request failed ({status})"),
            }));
        }

        let payload = response
            .json::<EmbedResponse>()
            .await
            .map_err(|err| SearchError::InvalidResponse(err.to_string()))?;

        if payload.embeddings.len() != input.len() {
            return Err(SearchError::InvalidResponse(format!(
                "expected {} embeddings, got {}",
                input.len(),
                payload.embeddings.len()
            )));
        }

        if payload
            .embeddings
            .iter()
            .any(|embedding| embedding.len() != self.dimensions)
        {
            return Err(SearchError::InvalidResponse(format!(
                "embedding dimension mismatch - expected {}",
                self.dimensions
            )));
        }

        Ok(payload.embeddings)
    }

    async fn acquire_local_permit(&self) -> Result<Option<OwnedSemaphorePermit>, SearchError> {
        match &self.ollama_semaphore {
            Some(semaphore) => semaphore
                .clone()
                .acquire_owned()
                .await
                .map(Some)
                .map_err(|err| SearchError::Request(err.to_string())),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Serialize)]
struct EmbedRequest<'a> {
    model: String,
    input: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TagsModel>,
}

#[derive(Debug, Deserialize)]
struct TagsModel {
    name: String,
}

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
            });
        }
    }

    chunks
}

pub fn chunk_transcript_content(
    content: &str,
    target_words: usize,
    overlap_words: usize,
) -> Vec<ChunkDraft> {
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
        })
        .collect()
}

pub fn fuse_ranked_matches(
    vector_ranks: &[(&str, usize)],
    fts_ranks: &[(&str, usize)],
    rrf_k: f32,
) -> Vec<(String, f32)> {
    let mut fused = HashMap::<String, f32>::new();

    for (chunk_id, rank) in vector_ranks.iter().chain(fts_ranks.iter()) {
        let score = 1.0 / (rrf_k + (*rank as f32));
        *fused.entry((*chunk_id).to_string()).or_default() += score;
    }

    let mut entries = fused.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    entries
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
    limit_text(text, MAX_SNIPPET_CHARS)
}

fn limit_error_detail(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    limit_text(&collapsed, MAX_ERROR_DETAIL_CHARS)
}

fn limit_text(text: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for character in text.chars().take(max_chars) {
        output.push(character);
    }

    if text.chars().count() > max_chars {
        output.push_str("...");
    }

    output
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
        SearchService, SearchSourceKind, chunk_summary_content, chunk_transcript_content,
        fuse_ranked_matches, hash_search_content,
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

        let chunks = chunk_transcript_content(&transcript, 12, 4);

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

        let chunks = chunk_transcript_content(transcript, 5, 0);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].text, "Alpha beta gamma delta.");
        assert_eq!(chunks[1].text, "Second paragraph starts here today.");
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
