use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

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

fn replace_markdown_links(input: &str) -> String {
    // Replace:
    // - [label](url) -> label
    // - ![alt](url) -> alt
    let mut out = String::with_capacity(input.len());
    let mut idx = 0usize;

    while idx < input.len() {
        let rest = &input[idx..];
        let next_img = rest.find("![");
        let next_link = rest.find('[');

        let next = match (next_img, next_link) {
            (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let Some(rel_start) = next else {
            out.push_str(&input[idx..]);
            break;
        };

        let start = idx + rel_start;
        out.push_str(&input[idx..start]);

        let after_start = &input[start..];
        if after_start.starts_with("![") {
            // image: ![alt](url)
            let label_start = start + 2; // skip ![
            let label_end = input[label_start..].find(']').map(|i| label_start + i);
            let Some(label_end) = label_end else {
                out.push('!');
                idx = start + 1;
                continue;
            };

            let after_bracket = &input[label_end + 1..];
            let Some(open_paren_rel) = after_bracket.find('(') else {
                out.push_str(&input[start..label_end + 1]);
                idx = label_end + 1;
                continue;
            };
            let open_paren = label_end + 1 + open_paren_rel;

            let after_paren = &input[open_paren + 1..];
            let Some(close_paren_rel) = after_paren.find(')') else {
                out.push_str(&input[start..open_paren]);
                idx = open_paren;
                continue;
            };
            let close_paren = open_paren + 1 + close_paren_rel;

            out.push_str(input[label_start..label_end].trim());
            idx = close_paren + 1;
            continue;
        }

        // normal link: [label](url)
        let label_start = start + 1; // skip [
        let label_end = input[label_start..].find(']').map(|i| label_start + i);
        let Some(label_end) = label_end else {
            out.push('[');
            idx = start + 1;
            continue;
        };

        let after_bracket = &input[label_end + 1..];
        let Some(open_paren_rel) = after_bracket.find('(') else {
            out.push_str(&input[start..label_end + 1]);
            idx = label_end + 1;
            continue;
        };
        let open_paren = label_end + 1 + open_paren_rel;

        let after_paren = &input[open_paren + 1..];
        let Some(close_paren_rel) = after_paren.find(')') else {
            out.push_str(&input[start..open_paren]);
            idx = open_paren;
            continue;
        };
        let close_paren = open_paren + 1 + close_paren_rel;

        out.push_str(input[label_start..label_end].trim());
        idx = close_paren + 1;
    }

    out
}

pub(crate) fn sanitize_markdown_for_tts(input: &str) -> String {
    const BREAK_AFTER_HEADING: &str = r#"<break time="0.6s" />"#;
    const BREAK_AFTER_LIST_ITEM: &str = r#"<break time="0.15s" />"#;

    let no_html = strip_html_tags(input);
    let links_stripped = replace_markdown_links(&no_html);

    let mut out = String::with_capacity(links_stripped.len());
    let mut in_fence = false;

    for line in links_stripped.lines() {
        let line_trim_start = line.trim_start();

        let trimmed = line.trim_start();

        // Toggle code fences, but keep the internal content (we just remove the markers).
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }

        let is_heading = line_trim_start.starts_with('#');
        let mut is_list_item = false;
        let mut is_ordered_list_item = false;

        let mut processed = line.to_string();

        // Headings: remove leading '#' and whitespace.
        let ts = processed.trim_start();
        if ts.starts_with('#') {
            processed = ts.trim_start_matches('#').trim_start().to_string();
        }

        // Blockquotes: remove leading '>' (and one following space).
        let pst = processed.trim_start();
        if pst.starts_with('>') {
            processed = pst[1..].trim_start().to_string();
        }

        // List prefixes.
        let pst2 = processed.trim_start();
        for prefix in ["- ", "* ", "+ "] {
            if pst2.starts_with(prefix) {
                processed = pst2[prefix.len()..].to_string();
                is_list_item = true;
                break;
            }
        }

        // Ordered list prefixes: "1. " / "1) "
        let pst3 = processed.trim_start();
        if pst3.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            let mut byte_idx = 0usize;
            for ch in pst3.chars() {
                if ch.is_ascii_digit() {
                    byte_idx += ch.len_utf8();
                } else {
                    break;
                }
            }
            let rest = &pst3[byte_idx..];
            if let Some(after) = rest.strip_prefix(". ") {
                processed = after.to_string();
                is_ordered_list_item = true;
            } else if let Some(after) = rest.strip_prefix(") ") {
                processed = after.to_string();
                is_ordered_list_item = true;
            }
        }

        if processed.is_empty() {
            continue;
        }

        // Remove inline emphasis/code/decorators and table separators.
        let decor_stripped = processed
            .chars()
            .filter(|ch| !matches!(*ch, '*' | '_' | '`' | '~' | '|' | '<' | '>'))
            .collect::<String>();

        let mut cleaned = decor_stripped.trim().to_string();
        if !cleaned.is_empty() {
            let ends_with_punctuation = cleaned
                .chars()
                .last()
                .is_some_and(|c| matches!(c, '.' | '!' | '?'));

            if is_heading {
                if !ends_with_punctuation {
                    cleaned.push('.');
                }
                cleaned.push(' ');
                cleaned.push_str(BREAK_AFTER_HEADING);
            } else if is_list_item || is_ordered_list_item {
                if !ends_with_punctuation {
                    cleaned.push('.');
                }
                cleaned.push(' ');
                cleaned.push_str(BREAK_AFTER_LIST_ITEM);
            }
        }

        if cleaned.is_empty() {
            continue;
        }

        // Add a space between lines rather than keeping raw newlines.
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&cleaned);

        // `in_fence` intentionally isn't used beyond fence marker skipping.
        let _ = in_fence;
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug)]
pub struct ElevenLabsTtsService {
    client: Client,
    api_key: String,
    configured_voice_id: Option<String>,
    resolved_voice_id: Mutex<Option<String>>,
    model_id: String,
    output_format: String,
}

#[derive(Debug, Error)]
pub enum ElevenLabsTtsError {
    #[error("summary text is empty")]
    EmptyText,
    #[error("failed to call ElevenLabs API: {0}")]
    Request(#[from] reqwest::Error),
    #[error("ElevenLabs API error ({status}): {body}")]
    ApiStatus { status: u16, body: String },
    #[error(
        "ElevenLabs voices_read permission is missing. Set ELEVENLABS_TTS_VOICE_ID (recommended) or grant `voices_read` to ELEVENLABS_TTS_API_KEY."
    )]
    MissingVoicesReadPermission,
    #[error("no ElevenLabs voices available for this account")]
    NoVoicesAvailable,
}

#[derive(Debug, Serialize)]
struct ElevenLabsTtsRequest<'a> {
    text: &'a str,
    model_id: &'a str,
    output_format: &'a str,
}

#[derive(Debug, Serialize)]
struct ElevenLabsTtsStreamRequest<'a> {
    text: &'a str,
    model_id: &'a str,
}

#[derive(Debug, Deserialize)]
struct ElevenLabsVoicesResponse {
    voices: Vec<ElevenLabsVoice>,
}

#[derive(Debug, Deserialize)]
struct ElevenLabsVoice {
    voice_id: String,
}

impl ElevenLabsTtsService {
    pub fn new(
        client: Client,
        api_key: String,
        voice_id: Option<String>,
        model_id: String,
        output_format: String,
    ) -> Self {
        Self {
            client,
            api_key,
            configured_voice_id: voice_id,
            resolved_voice_id: Mutex::new(None),
            model_id,
            output_format,
        }
    }

    pub async fn synthesize_summary(&self, text: &str) -> Result<Vec<u8>, ElevenLabsTtsError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ElevenLabsTtsError::EmptyText);
        }

        let voice_id = self.resolve_voice_id().await?;
        let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", voice_id);
        let payload = ElevenLabsTtsRequest {
            text,
            model_id: &self.model_id,
            output_format: &self.output_format,
        };

        let response = self
            .client
            .post(url)
            .header("xi-api-key", &self.api_key)
            .header("Accept", "audio/mpeg")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ElevenLabsTtsError::ApiStatus { status, body });
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn synthesize_summary_stream_response(
        &self,
        text: &str,
    ) -> Result<reqwest::Response, ElevenLabsTtsError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ElevenLabsTtsError::EmptyText);
        }

        let voice_id = self.resolve_voice_id().await?;
        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{voice_id}/stream?output_format={}&optimize_streaming_latency=2",
            self.output_format
        );
        let payload = ElevenLabsTtsStreamRequest {
            text,
            model_id: &self.model_id,
        };

        let response = self
            .client
            .post(url)
            .header("xi-api-key", &self.api_key)
            // ElevenLabs streams raw audio bytes; use a broad Accept so we don't
            // accidentally constrain formats (mp3, opus, pcm, etc.).
            .header("Accept", "application/octet-stream")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ElevenLabsTtsError::ApiStatus { status, body });
        }

        Ok(response)
    }

    async fn resolve_voice_id(&self) -> Result<String, ElevenLabsTtsError> {
        if let Some(voice_id) = self.configured_voice_id.as_ref() {
            return Ok(voice_id.clone());
        }

        {
            let cached = self.resolved_voice_id.lock().await;
            if let Some(voice_id) = cached.as_ref() {
                return Ok(voice_id.clone());
            }
        }

        let response = self
            .client
            .get("https://api.elevenlabs.io/v1/voices")
            .header("xi-api-key", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            if status == 401
                && (body.contains("voices_read")
                    || body.contains("missing_permissions")
                    || body.contains("permission"))
            {
                return Err(ElevenLabsTtsError::MissingVoicesReadPermission);
            }
            return Err(ElevenLabsTtsError::ApiStatus { status, body });
        }

        let body: ElevenLabsVoicesResponse = response.json().await?;
        let first_voice = body
            .voices
            .first()
            .map(|voice| voice.voice_id.clone())
            .ok_or(ElevenLabsTtsError::NoVoicesAvailable)?;

        let mut cached = self.resolved_voice_id.lock().await;
        *cached = Some(first_voice.clone());
        Ok(first_voice)
    }

    pub async fn resolve_voice_id_for_cache_key(&self) -> Result<String, ElevenLabsTtsError> {
        self.resolve_voice_id().await
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn output_format(&self) -> &str {
        &self.output_format
    }
}
