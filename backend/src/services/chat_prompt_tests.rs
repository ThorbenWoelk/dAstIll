use super::{
    append_reference_links, build_source_list_fallback_answer, build_tool_output_fallback_answer,
    is_model_availability_error,
};
use crate::models::{
    ChatSource, ContentItemKind, ContentPartKind, ContentSourceKind, ProviderKind,
};
use crate::search::SearchSourceKind;
use crate::services::chat::RetrievedChatSource;

#[test]
fn source_list_fallback_answer_keeps_citations_and_source_titles() {
    let answer = build_source_list_fallback_answer(
        "Find every video that mentions RAG.",
        &[RetrievedChatSource {
            source: ChatSource {
                source_id: "channel-1".to_string(),
                video_id: "video-1".to_string(),
                item_id: "video-1".to_string(),
                provider: ProviderKind::YouTube,
                content_source_kind: ContentSourceKind::YouTubeChannel,
                item_kind: ContentItemKind::Video,
                part_kind: ContentPartKind::Transcript,
                channel_id: "channel-1".to_string(),
                channel_name: "Channel One".to_string(),
                video_title: "RAG Patterns".to_string(),
                source_kind: SearchSourceKind::Transcript,
                section_title: None,
                snippet: "The speaker describes RAG retrieval and reranking.".to_string(),
                score: 1.0,
                chunk_id: "chunk-1".to_string(),
                retrieval_pass: Some(1),
            },
            context_text: "The speaker describes RAG retrieval and reranking.".to_string(),
        }],
    );

    assert!(answer.contains("RAG Patterns - Channel One"));
    assert!(answer.contains("[1]"));
    assert!(answer.contains("highest-ranked saved excerpts"));
}

#[test]
fn source_list_fallback_answer_lists_each_source_item_once() {
    let answer = build_source_list_fallback_answer(
        "What topics come up most across my library?",
        &[
            RetrievedChatSource {
                source: ChatSource {
                    source_id: "channel-1".to_string(),
                    video_id: "video-1".to_string(),
                    item_id: "video-1".to_string(),
                    provider: ProviderKind::YouTube,
                    content_source_kind: ContentSourceKind::YouTubeChannel,
                    item_kind: ContentItemKind::Video,
                    part_kind: ContentPartKind::GeneratedSummary,
                    channel_id: "channel-1".to_string(),
                    channel_name: "Channel One".to_string(),
                    video_title: "Same Video".to_string(),
                    source_kind: SearchSourceKind::Summary,
                    section_title: Some("TL;DR".to_string()),
                    snippet: "First excerpt.".to_string(),
                    score: 1.0,
                    chunk_id: "chunk-1".to_string(),
                    retrieval_pass: Some(1),
                },
                context_text: "First excerpt.".to_string(),
            },
            RetrievedChatSource {
                source: ChatSource {
                    source_id: "channel-1".to_string(),
                    video_id: "video-1".to_string(),
                    item_id: "video-1".to_string(),
                    provider: ProviderKind::YouTube,
                    content_source_kind: ContentSourceKind::YouTubeChannel,
                    item_kind: ContentItemKind::Video,
                    part_kind: ContentPartKind::Transcript,
                    channel_id: "channel-1".to_string(),
                    channel_name: "Channel One".to_string(),
                    video_title: "Same Video".to_string(),
                    source_kind: SearchSourceKind::Transcript,
                    section_title: None,
                    snippet: "Second excerpt.".to_string(),
                    score: 0.9,
                    chunk_id: "chunk-2".to_string(),
                    retrieval_pass: Some(1),
                },
                context_text: "Second excerpt.".to_string(),
            },
        ],
    );

    assert_eq!(answer.matches("Same Video - Channel One").count(), 1);
    assert!(answer.contains("First excerpt."));
    assert!(!answer.contains("Second excerpt."));
}

#[test]
fn source_list_fallback_answer_uses_contrast_language_for_comparisons() {
    let answer =
        build_source_list_fallback_answer("Which videos offer the strongest counterargument?", &[]);

    assert!(answer.contains("both"));
    assert!(answer.contains("while"));
    assert!(answer.contains("counterarguments"));
}

#[test]
fn tool_output_fallback_answer_keeps_tool_result_text() {
    let answer = build_tool_output_fallback_answer(
        "Show me all highlights related to search.",
        &["No saved highlights matched query \"search\".".to_string()],
    );

    assert!(answer.contains("tool results directly"));
    assert!(answer.contains("saved highlights"));
}

#[test]
fn source_list_fallback_answer_mentions_timestamps_for_navigation_prompts() {
    let answer = build_source_list_fallback_answer(
        "Find the section where the speaker gives an example.",
        &[],
    );

    assert!(answer.contains("timestamps"));
    assert!(answer.contains("section candidates"));
}

#[test]
fn source_list_fallback_answer_mentions_summary_and_transcript_for_alignment_prompts() {
    let answer = build_source_list_fallback_answer(
        "What evidence in the transcript supports the summary?",
        &[],
    );

    assert!(answer.contains("Summary/transcript alignment evidence"));
    assert!(answer.contains("summary"));
    assert!(answer.contains("transcript"));
}

#[test]
fn source_list_fallback_answer_adds_caveat_language_for_tone_prompts() {
    let answer = build_source_list_fallback_answer("What is the overall tone of this video?", &[]);

    assert!(answer.contains("From the available evidence"));
    assert!(answer.contains("tentative"));
}

#[test]
fn model_availability_error_matches_quota_and_cooldown_failures() {
    assert!(is_model_availability_error("429 Too Many Requests"));
    assert!(is_model_availability_error("cloud cooldown active"));
    assert!(is_model_availability_error("rate limited by provider"));
    assert!(!is_model_availability_error("Failed to parse stream line"));
}

#[test]
fn append_reference_links_lists_cited_sources_as_workspace_links() {
    let sources = vec![
        ChatSource {
            source_id: "channel-1".to_string(),
            video_id: "video-1".to_string(),
            item_id: "video-1".to_string(),
            provider: ProviderKind::YouTube,
            content_source_kind: ContentSourceKind::YouTubeChannel,
            item_kind: ContentItemKind::Video,
            part_kind: ContentPartKind::Transcript,
            channel_id: "channel-1".to_string(),
            channel_name: "Channel One".to_string(),
            video_title: "RAG Patterns".to_string(),
            source_kind: SearchSourceKind::Transcript,
            section_title: None,
            snippet: "The speaker describes RAG retrieval and reranking.".to_string(),
            score: 1.0,
            chunk_id: "chunk-1".to_string(),
            retrieval_pass: Some(1),
        },
        ChatSource {
            source_id: "channel-2".to_string(),
            video_id: "video-2".to_string(),
            item_id: "video-2".to_string(),
            provider: ProviderKind::YouTube,
            content_source_kind: ContentSourceKind::YouTubeChannel,
            item_kind: ContentItemKind::Video,
            part_kind: ContentPartKind::GeneratedSummary,
            channel_id: "channel-2".to_string(),
            channel_name: "Channel Two".to_string(),
            video_title: "Other Video".to_string(),
            source_kind: SearchSourceKind::Summary,
            section_title: None,
            snippet: "Other evidence.".to_string(),
            score: 0.5,
            chunk_id: "chunk-2".to_string(),
            retrieval_pass: Some(1),
        },
    ];

    let answer = append_reference_links("Use retrieval evidence.[1]".to_string(), &sources);

    assert!(answer.contains("References"));
    assert!(answer.contains("- [1] [RAG Patterns - Channel One](/?source=channel-1"));
    assert!(answer.contains("content=transcript"));
    assert!(!answer.contains("Other Video"));
}

#[test]
fn append_reference_links_groups_repeated_source_items() {
    let sources = vec![
        ChatSource {
            source_id: "channel-1".to_string(),
            video_id: "video-1".to_string(),
            item_id: "video-1".to_string(),
            provider: ProviderKind::YouTube,
            content_source_kind: ContentSourceKind::YouTubeChannel,
            item_kind: ContentItemKind::Video,
            part_kind: ContentPartKind::Transcript,
            channel_id: "channel-1".to_string(),
            channel_name: "Channel One".to_string(),
            video_title: "Same Video".to_string(),
            source_kind: SearchSourceKind::Transcript,
            section_title: None,
            snippet: "First evidence.".to_string(),
            score: 1.0,
            chunk_id: "chunk-1".to_string(),
            retrieval_pass: Some(1),
        },
        ChatSource {
            source_id: "channel-1".to_string(),
            video_id: "video-1".to_string(),
            item_id: "video-1".to_string(),
            provider: ProviderKind::YouTube,
            content_source_kind: ContentSourceKind::YouTubeChannel,
            item_kind: ContentItemKind::Video,
            part_kind: ContentPartKind::GeneratedSummary,
            channel_id: "channel-1".to_string(),
            channel_name: "Channel One".to_string(),
            video_title: "Same Video".to_string(),
            source_kind: SearchSourceKind::Summary,
            section_title: Some("Key Points".to_string()),
            snippet: "Second evidence.".to_string(),
            score: 0.9,
            chunk_id: "chunk-2".to_string(),
            retrieval_pass: Some(1),
        },
    ];

    let answer = append_reference_links("Use evidence.[1][2]".to_string(), &sources);

    assert_eq!(answer.matches("Same Video - Channel One").count(), 1);
    assert!(answer.contains("- [1, 2] [Same Video - Channel One]"));
}
