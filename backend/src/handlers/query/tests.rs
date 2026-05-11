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
        selected_source_id: Some("source-123".to_string()),
        selected_channel_id: Some("channel-123".to_string()),
        selected_item_id: Some("item-456".to_string()),
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
    assert_eq!(params.selected_source_id(), Some("source-123"));
}

#[test]
fn workspace_bootstrap_params_fall_back_to_channel_id_for_source_selection() {
    let params = WorkspaceBootstrapParams {
        selected_channel_id: Some("channel-123".to_string()),
        ..WorkspaceBootstrapParams::default()
    };

    assert_eq!(params.selected_source_id(), Some("channel-123"));
}
