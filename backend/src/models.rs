use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub handle: Option<String>,
    pub name: String,
    pub thumbnail_url: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    Pending,
    Loading,
    Ready,
    Failed,
}

impl ContentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }

    pub fn from_db_value(s: &str) -> Self {
        match s {
            "loading" => Self::Loading,
            "ready" => Self::Ready,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

impl std::str::FromStr for ContentStatus {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_db_value(s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub channel_id: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub is_short: bool,
    pub transcript_status: ContentStatus,
    pub summary_status: ContentStatus,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub video_id: String,
    pub watch_url: String,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub channel_name: Option<String>,
    pub channel_id: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub duration_iso8601: Option<String>,
    pub duration_seconds: Option<u64>,
    pub view_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub video_id: String,
    pub raw_text: Option<String>,
    pub formatted_markdown: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub video_id: String,
    pub content: String,
    pub model_used: Option<String>,
    pub quality_score: Option<u8>,
    pub quality_note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SummaryEvaluationJob {
    pub video_id: String,
    pub video_title: String,
    pub transcript_text: String,
    pub summary_content: String,
}

#[derive(Debug, Clone)]
pub struct SummaryEvaluationResult {
    pub quality_score: u8,
    pub quality_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddChannelRequest {
    pub input: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAcknowledgedRequest {
    pub acknowledged: bool,
}

#[derive(Debug, Serialize)]
pub struct CleanTranscriptResponse {
    pub content: String,
    pub preserved_text: bool,
}
