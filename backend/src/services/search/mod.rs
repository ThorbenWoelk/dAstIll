use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::services::http::build_http_client;
use crate::services::text::limit_text as limit_text_base;

pub const SEARCH_EMBEDDING_DIMENSIONS: usize = 512;
pub const SEARCH_TRANSCRIPT_TARGET_WORDS: usize = 300;
pub const SEARCH_TRANSCRIPT_OVERLAP_WORDS: usize = 40;
pub const SEARCH_SUMMARY_TARGET_WORDS: usize = 300;
// Re-export so callers don't need to know about the fusion module.
pub use crate::services::fusion::SEARCH_RRF_K;
pub use crate::services::fusion::fuse_ranked_matches;
const SEARCH_EMBED_BATCH_SIZE: usize = 8;
const SEARCH_EMBED_REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
const SEARCH_RERANK_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const SEARCH_HYDE_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Maximum candidates passed to the cross-encoder reranker per request.
const SEARCH_RERANK_MAX_CANDIDATES: usize = 50;
const MAX_ERROR_DETAIL_CHARS: usize = 240;
const MAX_SNIPPET_CHARS: usize = 420;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, ts_rs::TS)]
#[ts(export, export_to = "../../frontend/src/lib/bindings/")]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkDraft {
    pub source_kind: SearchSourceKind,
    pub section_title: Option<String>,
    pub text: String,
    pub word_count: usize,
    pub is_full_document: bool,
    /// Start position in the video (seconds). Only present for timed transcript chunks.
    pub start_sec: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct SearchIndexChunk {
    pub chunk_index: usize,
    pub section_title: Option<String>,
    pub chunk_text: String,
    pub embedding_json: Option<String>,
    pub token_count: usize,
    /// Start position in the video (seconds). Only present for timed transcript chunks.
    pub start_sec: Option<f32>,
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
    /// Start position in the video (seconds). Only present for timed transcript chunks.
    pub start_sec: Option<f32>,
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
    /// Optional cross-encoder reranker model (Ollama `/api/rerank`).
    rerank_model: Option<String>,
    /// Optional generative model for HyDE passage synthesis (Ollama `/api/generate`).
    hyde_model: Option<String>,
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
            rerank_model: None,
            hyde_model: None,
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
            rerank_model: None,
            hyde_model: None,
            api_key: None,
            dimensions,
            semantic_enabled,
            ollama_semaphore: None,
        }
    }

    pub fn with_rerank_model(mut self, model: Option<String>) -> Self {
        self.rerank_model = model;
        self
    }

    pub fn with_hyde_model(mut self, model: Option<String>) -> Self {
        self.hyde_model = model;
        self
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

    pub fn rerank_model(&self) -> Option<&str> {
        self.rerank_model.as_deref()
    }

    pub fn hyde_model(&self) -> Option<&str> {
        self.hyde_model.as_deref()
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

    /// Re-rank candidates using a cross-encoder model via Ollama `/api/rerank`.
    /// Falls back to the original order if the reranker is not configured or the call fails.
    pub async fn rerank_candidates(
        &self,
        query: &str,
        candidates: Vec<SearchCandidate>,
    ) -> Result<Vec<SearchCandidate>, SearchError> {
        let Some(model) = self.rerank_model.as_ref() else {
            return Ok(candidates);
        };
        if candidates.len() <= 1 {
            return Ok(candidates);
        }

        let top: Vec<SearchCandidate> = candidates
            .into_iter()
            .take(SEARCH_RERANK_MAX_CANDIDATES)
            .collect();
        let documents: Vec<String> = top.iter().map(|c| c.chunk_text.clone()).collect();

        let _permit = self.acquire_local_permit().await?;
        let response = self
            .auth(
                self.client
                    .post(format!("{}/api/rerank", self.base_url))
                    .timeout(SEARCH_RERANK_REQUEST_TIMEOUT)
                    .json(&RerankRequest {
                        model: model.as_str(),
                        query,
                        documents: &documents,
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
                .map(|t| limit_error_detail(&t))
                .filter(|t| !t.is_empty());
            return Err(SearchError::Request(match detail {
                Some(d) => format!("Ollama rerank request failed ({status}): {d}"),
                None => format!("Ollama rerank request failed ({status})"),
            }));
        }

        let payload = response
            .json::<RerankResponse>()
            .await
            .map_err(|err| SearchError::InvalidResponse(err.to_string()))?;

        let mut scored: Vec<(usize, f32)> = payload
            .results
            .into_iter()
            .map(|r| (r.index, r.relevance_score))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .filter_map(|(idx, _)| top.get(idx).cloned())
            .collect())
    }

    /// Generate a hypothetical document passage for HyDE using Ollama `/api/generate`.
    /// Short queries (≤4 meaningful tokens) benefit from a richer embedding target.
    pub async fn generate_hyde_passage(&self, query: &str) -> Result<String, SearchError> {
        let Some(model) = self.hyde_model.as_ref() else {
            return Err(SearchError::Request(
                "HyDE model not configured".to_string(),
            ));
        };

        let prompt = format!(
            "Write a concise 2-3 sentence passage that directly answers: \"{query}\". \
             Be specific. Output only the passage, nothing else."
        );

        let _permit = self.acquire_local_permit().await?;
        let response = self
            .auth(
                self.client
                    .post(format!("{}/api/generate", self.base_url))
                    .timeout(SEARCH_HYDE_REQUEST_TIMEOUT)
                    .json(&HydeRequest {
                        model: model.as_str(),
                        prompt: &prompt,
                        stream: false,
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
                .map(|t| limit_error_detail(&t))
                .filter(|t| !t.is_empty());
            return Err(SearchError::Request(match detail {
                Some(d) => format!("Ollama HyDE request failed ({status}): {d}"),
                None => format!("Ollama HyDE request failed ({status})"),
            }));
        }

        let payload = response
            .json::<HydeResponse>()
            .await
            .map_err(|err| SearchError::InvalidResponse(err.to_string()))?;

        let passage = payload.response.trim().to_string();
        if passage.is_empty() {
            return Err(SearchError::InvalidResponse(
                "HyDE generated empty passage".to_string(),
            ));
        }
        Ok(passage)
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

#[derive(Debug, Serialize)]
struct RerankRequest<'a> {
    model: &'a str,
    query: &'a str,
    documents: &'a [String],
}

#[derive(Debug, Deserialize)]
struct RerankResult {
    index: usize,
    relevance_score: f32,
}

#[derive(Debug, Deserialize)]
struct RerankResponse {
    results: Vec<RerankResult>,
}

#[derive(Debug, Serialize)]
struct HydeRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct HydeResponse {
    response: String,
}


include!("frag_02.rs");
