use serde::Deserialize;

use crate::db;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VideoTypeFilter {
    All,
    Long,
    Short,
}

impl VideoTypeFilter {
    pub fn as_is_short(self) -> Option<bool> {
        match self {
            Self::All => None,
            Self::Long => Some(false),
            Self::Short => Some(true),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueueTab {
    Transcripts,
    Summaries,
    Evaluations,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct VideoListParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_shorts: Option<bool>,
    pub video_type: Option<VideoTypeFilter>,
    pub acknowledged: Option<bool>,
    pub queue_only: Option<bool>,
    pub queue_tab: Option<QueueTab>,
}

impl VideoListParams {
    pub fn limit_or_default(&self) -> usize {
        self.limit.unwrap_or(20).min(100)
    }

    pub fn offset_or_default(&self) -> usize {
        self.offset.unwrap_or(0)
    }

    pub fn is_short_filter(&self) -> Option<bool> {
        match self.video_type {
            Some(video_type) => video_type.as_is_short(),
            None => {
                if self.include_shorts.unwrap_or(true) {
                    None
                } else {
                    Some(false)
                }
            }
        }
    }

    pub fn acknowledged_filter(&self) -> Option<bool> {
        self.acknowledged
    }

    pub fn queue_filter(&self) -> Option<db::QueueFilter> {
        match self.queue_tab {
            Some(QueueTab::Transcripts) => Some(db::QueueFilter::TranscriptsOnly),
            Some(QueueTab::Summaries) => Some(db::QueueFilter::SummariesOnly),
            Some(QueueTab::Evaluations) => Some(db::QueueFilter::EvaluationsOnly),
            None if self.queue_only.unwrap_or(false) => Some(db::QueueFilter::AnyIncomplete),
            None => None,
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct WorkspaceBootstrapParams {
    pub selected_channel_id: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub include_shorts: Option<bool>,
    pub video_type: Option<VideoTypeFilter>,
    pub acknowledged: Option<bool>,
    pub queue_only: Option<bool>,
    pub queue_tab: Option<QueueTab>,
}

impl WorkspaceBootstrapParams {
    pub fn video_params(&self) -> VideoListParams {
        VideoListParams {
            limit: self.limit,
            offset: self.offset,
            include_shorts: self.include_shorts,
            video_type: self.video_type,
            acknowledged: self.acknowledged,
            queue_only: self.queue_only,
            queue_tab: self.queue_tab,
        }
    }
}
