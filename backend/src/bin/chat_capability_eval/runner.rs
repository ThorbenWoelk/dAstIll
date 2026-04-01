use std::env;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use dastill::models::ChatConversation;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};

use crate::grading::grade_prompt_result;
use crate::model::{
    DEFAULT_PROXY_TOKEN, FAILURE_STREAM, PromptRunResult, PromptRunStatus, PromptSpec, SweepRunner,
};
use crate::sse::{
    ParsedStream, SseAccumulator, parse_done_event, parse_error_event, parse_sources_event,
    parse_status_event, parse_token_event,
};

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
        default_headers.insert("x-dastill-role", HeaderValue::from_static("operator"));
        default_headers.insert("x-dastill-client-ip", HeaderValue::from_static("127.0.0.1"));

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            default_headers,
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn create_conversation(&self) -> Result<ChatConversation> {
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
            bail!("conversation create failed: {status} {body}");
        }

        response
            .json::<ChatConversation>()
            .await
            .context("failed to decode conversation create response")
    }

    async fn send_prompt(
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

    pub(crate) async fn run_prompt(
        &self,
        spec: &PromptSpec,
        deep_research: bool,
        model: Option<&str>,
    ) -> Result<PromptRunResult> {
        let started_at = Instant::now();
        let conversation = self.create_conversation().await?;
        let conversation_id = conversation.id.clone();

        let stream = self
            .send_prompt(&conversation_id, &spec.prompt, deep_research, model)
            .await;

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
                prompt: spec.prompt.clone(),
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
}
