use std::env;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use dastill::models::ChatConversation;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::grading::grade_prompt_result;
use crate::model::{
    DEFAULT_PROXY_TOKEN, FAILURE_STREAM, PromptRunResult, PromptRunStatus, PromptSpec, SweepRunner,
};
use crate::sse::{
    ParsedStream, SseAccumulator, parse_done_event, parse_error_event, parse_sources_event,
    parse_status_event, parse_token_event,
};

enum ConversationBootstrap {
    Persistent(ChatConversation),
    Ephemeral(ChatConversation),
}

#[derive(Debug, Deserialize)]
struct SuggestionItem {
    id: String,
    label: String,
}

#[derive(Debug, Deserialize)]
struct VideoStatusProbe {
    transcript_status: String,
    summary_status: String,
}

impl SweepRunner {
    pub(crate) fn new(base_url: &str, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .context("failed to build HTTP client")?;
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        default_headers.insert(
            "x-dastill-proxy-auth",
            HeaderValue::from_str(
                &env::var("BACKEND_PROXY_TOKEN")
                    .unwrap_or_else(|_| DEFAULT_PROXY_TOKEN.to_string()),
            )
            .context("invalid BACKEND_PROXY_TOKEN header value")?,
        );
        default_headers.insert("x-dastill-client-ip", HeaderValue::from_static("127.0.0.1"));
        apply_eval_identity_headers(
            &mut default_headers,
            env::var("CHAT_EVAL_USER_ID").ok().as_deref(),
            env::var("CHAT_EVAL_ROLE").ok().as_deref(),
        )?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            default_headers,
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn bootstrap_conversation(&self) -> Result<ConversationBootstrap> {
        let response = self
            .client
            .post(self.api_url("/api/chat/conversations"))
            .headers(self.default_headers.clone())
            .json(&serde_json::json!({ "title": null }))
            .send()
            .await
            .context("failed to create conversation")?;

        if response.status() != StatusCode::CREATED {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if persistent_chat_requires_ephemeral(status.as_u16(), &body) {
                return Ok(ConversationBootstrap::Ephemeral(
                    create_ephemeral_conversation(),
                ));
            }
            bail!("conversation create failed: {status} {body}");
        }

        response
            .json::<ChatConversation>()
            .await
            .map(ConversationBootstrap::Persistent)
            .context("failed to decode conversation create response")
    }

    async fn send_persistent_prompt(
        &self,
        conversation_id: &str,
        prompt: &str,
        deep_research: bool,
        model: Option<&str>,
    ) -> Result<ParsedStream> {
        let mut request_body = serde_json::json!({
            "content": prompt,
            "deep_research": deep_research,
        });
        if let Some(model) = model {
            request_body["model"] = serde_json::Value::String(model.to_string());
        }

        let mut headers = self.default_headers.clone();
        headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));

        let response = self
            .client
            .post(self.api_url(&format!(
                "/api/chat/conversations/{conversation_id}/messages"
            )))
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .context("failed to start chat stream")?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("chat stream start failed: {status} {body}");
        }

        let started_at = Instant::now();
        let mut response = response;
        let mut parser = SseAccumulator::default();
        let mut raw_sse = String::new();
        let mut statuses = Vec::new();
        let mut latest_sources = Vec::new();
        let mut final_message = None;
        let mut error_message = None;

        while let Some(chunk) = response
            .chunk()
            .await
            .context("failed to read stream chunk")?
        {
            let text = String::from_utf8_lossy(&chunk);
            raw_sse.push_str(&text);
            for event in parser.push(&text) {
                let received_at_ms = started_at.elapsed().as_millis() as u64;
                match event.name.as_str() {
                    "status" => {
                        statuses.push(parse_status_event(received_at_ms, &event.data)?);
                    }
                    "sources" => {
                        latest_sources = parse_sources_event(&event.data)?;
                    }
                    "token" => {
                        let _ = parse_token_event(&event.data)?;
                    }
                    "done" => {
                        final_message = Some(parse_done_event(&event.data)?);
                    }
                    "error" => {
                        error_message = Some(parse_error_event(&event.data)?);
                    }
                    _ => {}
                }
            }
        }

        for event in parser.finish() {
            let received_at_ms = started_at.elapsed().as_millis() as u64;
            match event.name.as_str() {
                "status" => {
                    statuses.push(parse_status_event(received_at_ms, &event.data)?);
                }
                "sources" => {
                    latest_sources = parse_sources_event(&event.data)?;
                }
                "token" => {
                    let _ = parse_token_event(&event.data)?;
                }
                "done" => {
                    final_message = Some(parse_done_event(&event.data)?);
                }
                "error" => {
                    error_message = Some(parse_error_event(&event.data)?);
                }
                _ => {}
            }
        }

        Ok(ParsedStream {
            statuses,
            latest_sources,
            final_message,
            error_message,
            raw_sse,
        })
    }

    async fn send_ephemeral_prompt(
        &self,
        conversation: &ChatConversation,
        prompt: &str,
        deep_research: bool,
        model: Option<&str>,
    ) -> Result<ParsedStream> {
        let mut request_body = serde_json::json!({
            "conversation": conversation,
            "content": prompt,
            "deep_research": deep_research,
        });
        if let Some(model) = model {
            request_body["model"] = serde_json::Value::String(model.to_string());
        }

        let mut headers = self.default_headers.clone();
        headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));

        let response = self
            .client
            .post(self.api_url("/api/chat/ephemeral/messages"))
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .context("failed to start ephemeral chat stream")?;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("ephemeral chat stream start failed: {status} {body}");
        }

        self.consume_stream(response).await
    }

    async fn consume_stream(&self, mut response: reqwest::Response) -> Result<ParsedStream> {
        let started_at = Instant::now();
        let mut parser = SseAccumulator::default();
        let mut raw_sse = String::new();
        let mut statuses = Vec::new();
        let mut latest_sources = Vec::new();
        let mut final_message = None;
        let mut error_message = None;

        while let Some(chunk) = response
            .chunk()
            .await
            .context("failed to read stream chunk")?
        {
            let text = String::from_utf8_lossy(&chunk);
            raw_sse.push_str(&text);
            for event in parser.push(&text) {
                let received_at_ms = started_at.elapsed().as_millis() as u64;
                match event.name.as_str() {
                    "status" => {
                        statuses.push(parse_status_event(received_at_ms, &event.data)?);
                    }
                    "sources" => {
                        latest_sources = parse_sources_event(&event.data)?;
                    }
                    "token" => {
                        let _ = parse_token_event(&event.data)?;
                    }
                    "done" => {
                        final_message = Some(parse_done_event(&event.data)?);
                    }
                    "error" => {
                        error_message = Some(parse_error_event(&event.data)?);
                    }
                    _ => {}
                }
            }
        }

        for event in parser.finish() {
            let received_at_ms = started_at.elapsed().as_millis() as u64;
            match event.name.as_str() {
                "status" => {
                    statuses.push(parse_status_event(received_at_ms, &event.data)?);
                }
                "sources" => {
                    latest_sources = parse_sources_event(&event.data)?;
                }
                "token" => {
                    let _ = parse_token_event(&event.data)?;
                }
                "done" => {
                    final_message = Some(parse_done_event(&event.data)?);
                }
                "error" => {
                    error_message = Some(parse_error_event(&event.data)?);
                }
                _ => {}
            }
        }

        Ok(ParsedStream {
            statuses,
            latest_sources,
            final_message,
            error_message,
            raw_sse,
        })
    }

    pub(crate) async fn run_prompt(
        &self,
        spec: &PromptSpec,
        deep_research: bool,
        model: Option<&str>,
    ) -> Result<PromptRunResult> {
        let started_at = Instant::now();
        let resolved_prompt = self.resolve_prompt_scope(&spec.prompt).await?;
        let bootstrap = self.bootstrap_conversation().await?;
        let conversation = match &bootstrap {
            ConversationBootstrap::Persistent(conversation)
            | ConversationBootstrap::Ephemeral(conversation) => conversation.clone(),
        };
        let conversation_id = conversation.id.clone();

        let stream = match &bootstrap {
            ConversationBootstrap::Persistent(_) => {
                self.send_persistent_prompt(
                    &conversation_id,
                    &resolved_prompt,
                    deep_research,
                    model,
                )
                .await
            }
            ConversationBootstrap::Ephemeral(conversation) => {
                self.send_ephemeral_prompt(conversation, &resolved_prompt, deep_research, model)
                    .await
            }
        };

        let total_ms = started_at.elapsed().as_millis() as u64;

        match stream {
            Ok(parsed) => Ok(grade_prompt_result(
                spec,
                Some(conversation_id),
                parsed,
                total_ms,
            )),
            Err(error) => Ok(PromptRunResult {
                prompt_id: spec.id.clone(),
                prompt: resolved_prompt,
                capability_class: spec.capability_class,
                answerability_expected: spec.answerability_expected,
                conversation_id: Some(conversation_id),
                status: PromptRunStatus::HttpError,
                assistant_content: String::new(),
                source_count: 0,
                source_videos: Vec::new(),
                source_channels: Vec::new(),
                used_search_tool: false,
                used_db_tool: false,
                used_conversation_only: false,
                status_trace: Vec::new(),
                tool_calls: Vec::new(),
                latency_ms_total: total_ms,
                latency_ms_retrieval: None,
                latency_ms_generation: None,
                rubric_answerability_pass: false,
                rubric_grounding_pass: false,
                rubric_shape_pass: false,
                rubric_capability_score: 0,
                failure_class: Some(FAILURE_STREAM.to_string()),
                notes: vec!["failed to obtain a complete stream".to_string()],
                raw_error: Some(error.to_string()),
                raw_sse: None,
            }),
        }
    }

    async fn resolve_prompt_scope(&self, prompt: &str) -> Result<String> {
        if has_explicit_scope(prompt) {
            return Ok(prompt.trim().to_string());
        }

        if prompt_refers_to_current_video(prompt)
            && let Some(label) = self.fetch_top_processed_video_suggestion().await?
        {
            return Ok(format!("+{{{label}}} {}", prompt.trim()));
        }

        if prompt_refers_to_current_channel(prompt)
            && let Some(label) = self
                .fetch_top_suggestion("/api/chat/suggestions/channels")
                .await?
        {
            return Ok(format!("@{{{label}}} {}", prompt.trim()));
        }

        Ok(prompt.trim().to_string())
    }

    async fn fetch_top_processed_video_suggestion(&self) -> Result<Option<String>> {
        let response = self
            .client
            .get(self.api_url("/api/chat/suggestions/videos?limit=10"))
            .headers(self.default_headers.clone())
            .send()
            .await
            .context("failed to fetch video suggestions")?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let items = response
            .json::<Vec<SuggestionItem>>()
            .await
            .context("failed to decode video suggestions")?;

        for item in items {
            if self.video_has_ready_content(&item.id).await? {
                return Ok(Some(item.label));
            }
        }

        Ok(None)
    }

    async fn fetch_top_suggestion(&self, path: &str) -> Result<Option<String>> {
        let response = self
            .client
            .get(self.api_url(&format!("{path}?limit=1")))
            .headers(self.default_headers.clone())
            .send()
            .await
            .with_context(|| format!("failed to fetch suggestion from {path}"))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let items = response
            .json::<Vec<SuggestionItem>>()
            .await
            .with_context(|| format!("failed to decode suggestion response from {path}"))?;
        Ok(items.into_iter().next().map(|item| item.label))
    }

    async fn video_has_ready_content(&self, video_id: &str) -> Result<bool> {
        let response = self
            .client
            .get(self.api_url(&format!("/api/videos/{video_id}")))
            .headers(self.default_headers.clone())
            .send()
            .await
            .with_context(|| format!("failed to probe video status for {video_id}"))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let video = response
            .json::<VideoStatusProbe>()
            .await
            .with_context(|| format!("failed to decode video status for {video_id}"))?;

        Ok(video.summary_status == "ready" || video.transcript_status == "ready")
    }
}

fn create_ephemeral_conversation() -> ChatConversation {
    let now = chrono::Utc::now();
    ChatConversation {
        id: format!("conv_eval_{}", now.timestamp_millis()),
        title: None,
        title_status: dastill::models::ChatTitleStatus::Idle,
        created_at: now,
        updated_at: now,
        messages: Vec::new(),
    }
}

pub(crate) fn persistent_chat_requires_ephemeral(status: u16, body: &str) -> bool {
    status == StatusCode::FORBIDDEN.as_u16()
        && body
            .to_ascii_lowercase()
            .contains("signed-out chat stays ephemeral")
}

pub(crate) fn apply_eval_identity_headers(
    headers: &mut HeaderMap,
    user_id: Option<&str>,
    role: Option<&str>,
) -> Result<()> {
    let Some(user_id) = user_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    headers.insert(
        "x-dastill-auth-state",
        HeaderValue::from_static("authenticated"),
    );
    headers.insert(
        "x-dastill-user-id",
        HeaderValue::from_str(user_id).context("invalid CHAT_EVAL_USER_ID header value")?,
    );
    headers.insert(
        "x-dastill-role",
        HeaderValue::from_str(
            role.map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("user"),
        )
        .context("invalid CHAT_EVAL_ROLE header value")?,
    );
    Ok(())
}

pub(crate) fn has_explicit_scope(prompt: &str) -> bool {
    prompt.contains("@{")
        || prompt.contains("+{")
        || prompt.contains("@\"")
        || prompt.contains("+\"")
}

pub(crate) fn prompt_refers_to_current_video(prompt: &str) -> bool {
    let normalized = prompt.to_ascii_lowercase();
    ["this video", "the video", "this transcript"]
        .iter()
        .any(|needle| normalized.contains(needle))
}

pub(crate) fn prompt_refers_to_current_channel(prompt: &str) -> bool {
    let normalized = prompt.to_ascii_lowercase();
    ["this creator", "this channel"]
        .iter()
        .any(|needle| normalized.contains(needle))
}
