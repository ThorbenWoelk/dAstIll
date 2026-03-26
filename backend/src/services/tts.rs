use reqwest::Client;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ElevenLabsTtsService {
    client: Client,
    api_key: String,
    voice_id: String,
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
}

#[derive(Debug, Serialize)]
struct ElevenLabsTtsRequest<'a> {
    text: &'a str,
    model_id: &'a str,
    output_format: &'a str,
}

impl ElevenLabsTtsService {
    pub fn new(
        client: Client,
        api_key: String,
        voice_id: String,
        model_id: String,
        output_format: String,
    ) -> Self {
        Self {
            client,
            api_key,
            voice_id,
            model_id,
            output_format,
        }
    }

    pub async fn synthesize_summary(&self, text: &str) -> Result<Vec<u8>, ElevenLabsTtsError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ElevenLabsTtsError::EmptyText);
        }

        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}",
            self.voice_id
        );
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
}
