use crate::model::{
    DoneEventPayload, ErrorEventPayload, SourcesEventPayload, StreamStatusPayload, TimedStatus,
    TokenEventPayload,
};
use dastill::models::{ChatMessage, ChatSource};

pub(crate) fn parse_sse_block(block: &str) -> Option<SseEvent> {
    let mut event_name = None::<String>;
    let mut data_lines = Vec::new();
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        }
    }
    let name = event_name?;
    let data = data_lines.join("\n");
    Some(SseEvent { name, data })
}

pub(crate) fn parse_status_event(received_at_ms: u64, data: &str) -> anyhow::Result<TimedStatus> {
    let payload = serde_json::from_str::<StreamStatusPayload>(data)
        .map_err(anyhow::Error::from)
        .map_err(|error| error.context(format!("failed to parse status payload: {data}")))?;
    Ok(TimedStatus {
        received_at_ms,
        payload,
    })
}

pub(crate) fn parse_sources_event(data: &str) -> anyhow::Result<Vec<ChatSource>> {
    let payload = serde_json::from_str::<SourcesEventPayload>(data)
        .map_err(anyhow::Error::from)
        .map_err(|error| error.context(format!("failed to parse sources payload: {data}")))?;
    Ok(payload.sources)
}

pub(crate) fn parse_token_event(data: &str) -> anyhow::Result<String> {
    let payload = serde_json::from_str::<TokenEventPayload>(data)
        .map_err(anyhow::Error::from)
        .map_err(|error| error.context(format!("failed to parse token payload: {data}")))?;
    Ok(payload.token)
}

pub(crate) fn parse_done_event(data: &str) -> anyhow::Result<ChatMessage> {
    let payload = serde_json::from_str::<DoneEventPayload>(data)
        .map_err(anyhow::Error::from)
        .map_err(|error| error.context(format!("failed to parse done payload: {data}")))?;
    Ok(payload.message)
}

pub(crate) fn parse_error_event(data: &str) -> anyhow::Result<String> {
    let payload = serde_json::from_str::<ErrorEventPayload>(data)
        .map_err(anyhow::Error::from)
        .map_err(|error| error.context(format!("failed to parse error payload: {data}")))?;
    Ok(payload.message)
}

#[derive(Debug)]
pub(crate) struct SseEvent {
    pub(crate) name: String,
    pub(crate) data: String,
}

#[derive(Debug, Default)]
pub(crate) struct SseAccumulator {
    buffer: String,
}

#[derive(Debug)]
pub(crate) struct ParsedStream {
    pub(crate) statuses: Vec<TimedStatus>,
    pub(crate) latest_sources: Vec<ChatSource>,
    pub(crate) final_message: Option<ChatMessage>,
    pub(crate) error_message: Option<String>,
    pub(crate) raw_sse: String,
}

impl SseAccumulator {
    pub(crate) fn push(&mut self, chunk: &str) -> Vec<SseEvent> {
        self.buffer.push_str(&chunk.replace('\r', ""));
        let mut events = Vec::new();
        while let Some(index) = self.buffer.find("\n\n") {
            let block = self.buffer[..index].to_string();
            self.buffer.drain(..index + 2);
            if let Some(event) = parse_sse_block(&block) {
                events.push(event);
            }
        }
        events
    }

    pub(crate) fn finish(&mut self) -> Vec<SseEvent> {
        if self.buffer.trim().is_empty() {
            return Vec::new();
        }
        let block = std::mem::take(&mut self.buffer);
        parse_sse_block(&block).into_iter().collect()
    }
}
