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

#[cfg(test)]
mod tests {
    use super::{QueueTab, VideoListParams, VideoTypeFilter, WorkspaceBootstrapParams};
    use crate::db::QueueFilter;

    #[test]
    fn video_type_filter_overrides_include_shorts() {
        let params = VideoListParams {
            include_shorts: Some(true),
            video_type: Some(VideoTypeFilter::Long),
            ..VideoListParams::default()
        };

        assert_eq!(params.is_short_filter(), Some(false));
    }

    #[test]
    fn include_shorts_false_filters_out_shorts_when_video_type_is_missing() {
        let params = VideoListParams {
            include_shorts: Some(false),
            ..VideoListParams::default()
        };

        assert_eq!(params.is_short_filter(), Some(false));
    }

    #[test]
    fn queue_tab_maps_to_specific_queue_filter() {
        let params = VideoListParams {
            queue_only: Some(false),
            queue_tab: Some(QueueTab::Summaries),
            ..VideoListParams::default()
        };

        assert_eq!(params.queue_filter(), Some(QueueFilter::SummariesOnly));
    }

    #[test]
    fn queue_only_without_tab_maps_to_any_incomplete() {
        let params = VideoListParams {
            queue_only: Some(true),
            ..VideoListParams::default()
        };

        assert_eq!(params.queue_filter(), Some(QueueFilter::AnyIncomplete));
    }

    #[test]
    fn workspace_bootstrap_params_preserve_video_filters() {
        let params = WorkspaceBootstrapParams {
            selected_channel_id: Some("channel-123".to_string()),
            limit: Some(30),
            offset: Some(5),
            include_shorts: Some(false),
            video_type: Some(VideoTypeFilter::Short),
            acknowledged: Some(true),
            queue_only: Some(true),
            queue_tab: Some(QueueTab::Transcripts),
        };
        let video_params = params.video_params();

        assert_eq!(video_params.limit, Some(30));
        assert_eq!(video_params.offset, Some(5));
        assert_eq!(video_params.include_shorts, Some(false));
        assert_eq!(video_params.video_type, Some(VideoTypeFilter::Short));
        assert_eq!(video_params.acknowledged, Some(true));
        assert_eq!(video_params.queue_only, Some(true));
        assert_eq!(video_params.queue_tab, Some(QueueTab::Transcripts));
    }
}
