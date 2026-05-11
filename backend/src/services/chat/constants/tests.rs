use chrono::Utc;

use super::{
    CHAT_CONVERSATION_MAX_MESSAGES, CHAT_CONVERSATION_MAX_TOTAL_CHARS, CHAT_MESSAGE_MAX_CHARS,
    CHAT_MESSAGE_MAX_SOURCES, CHAT_SYSTEM_PROMPT, CHAT_SYSTEM_PROMPT_CONVERSATION_TURN,
    CHAT_VIDEO_OBSERVATION_PROMPT, enforce_chat_conversation_storage_limits,
    validate_chat_conversation_bounds, validate_chat_prompt, validate_chat_title_length,
};
use crate::models::{ChatConversation, ChatMessage, ChatMessageStatus, ChatRole, ChatSource};
use crate::search::SearchSourceKind;

fn message(role: ChatRole, content: &str) -> ChatMessage {
    ChatMessage {
        id: format!("msg-{}", content.len()),
        role,
        content: content.to_string(),
        sources: Vec::new(),
        status: ChatMessageStatus::Completed,
        created_at: Utc::now(),
        model: None,
        prompt_tokens: None,
        completion_tokens: None,
        total_duration_ns: None,
        turn_trace: None,
    }
}

fn source(index: usize) -> ChatSource {
    ChatSource {
        source_id: format!("channel-{index}"),
        video_id: format!("video-{index}"),
        item_id: format!("video-{index}"),
        provider: crate::models::ProviderKind::YouTube,
        content_source_kind: crate::models::ContentSourceKind::YouTubeChannel,
        item_kind: crate::models::ContentItemKind::Video,
        part_kind: crate::models::ContentPartKind::GeneratedSummary,
        channel_id: format!("channel-{index}"),
        channel_name: format!("Channel {index}"),
        video_title: format!("Video {index}"),
        source_kind: SearchSourceKind::Summary,
        section_title: Some(format!("Section {index}")),
        snippet: format!("Snippet {index}"),
        score: 1.0,
        chunk_id: format!("chunk-{index}"),
        retrieval_pass: None,
    }
}

fn conversation(messages: Vec<ChatMessage>) -> ChatConversation {
    ChatConversation {
        id: "conv-1".to_string(),
        title: None,
        title_status: crate::models::ChatTitleStatus::Idle,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        messages,
    }
}

#[test]
fn validate_chat_prompt_rejects_oversized_input() {
    let prompt = "x".repeat(CHAT_MESSAGE_MAX_CHARS + 1);
    assert_eq!(
        validate_chat_prompt(&prompt).expect_err("oversized prompt should fail"),
        "Message content is too large."
    );
}

#[test]
fn validate_chat_title_length_rejects_oversized_input() {
    let title = "x".repeat(super::CHAT_TITLE_MAX_CHARS + 1);
    assert_eq!(
        validate_chat_title_length(&title).expect_err("oversized title should fail"),
        "Conversation title is too large."
    );
}

#[test]
fn validate_chat_conversation_bounds_rejects_too_many_messages() {
    let messages = (0..=CHAT_CONVERSATION_MAX_MESSAGES)
        .map(|index| message(ChatRole::User, &format!("message {index}")))
        .collect();
    let conversation = conversation(messages);
    assert_eq!(
        validate_chat_conversation_bounds(&conversation)
            .expect_err("oversized conversation should fail"),
        "Conversation has too many messages for one request."
    );
}

#[test]
fn validate_chat_conversation_bounds_rejects_too_many_sources() {
    let mut message = message(ChatRole::Assistant, "answer");
    message.sources = (0..=CHAT_MESSAGE_MAX_SOURCES).map(source).collect();
    let conversation = conversation(vec![message]);
    assert_eq!(
        validate_chat_conversation_bounds(&conversation)
            .expect_err("oversized source list should fail"),
        "Conversation contains too many sources in one message."
    );
}

#[test]
fn enforce_chat_conversation_storage_limits_drops_oldest_messages() {
    let messages = (0..(CHAT_CONVERSATION_MAX_MESSAGES + 3))
        .map(|index| message(ChatRole::User, &format!("message {index}")))
        .collect();
    let mut conversation = conversation(messages);

    enforce_chat_conversation_storage_limits(&mut conversation);

    assert_eq!(conversation.messages.len(), CHAT_CONVERSATION_MAX_MESSAGES);
    assert_eq!(conversation.messages[0].content, "message 3");
}

#[test]
fn enforce_chat_conversation_storage_limits_trims_to_total_budget() {
    let mut conversation = conversation(vec![
        message(
            ChatRole::User,
            &"a".repeat(CHAT_CONVERSATION_MAX_TOTAL_CHARS),
        ),
        message(ChatRole::Assistant, "latest"),
    ]);

    enforce_chat_conversation_storage_limits(&mut conversation);

    assert_eq!(conversation.messages.len(), 1);
    assert_eq!(conversation.messages[0].content, "latest");
}

#[test]
fn answer_prompts_ban_emojis() {
    assert!(CHAT_SYSTEM_PROMPT.contains("Do not use emojis anywhere in the answer."));
    assert!(
        CHAT_SYSTEM_PROMPT_CONVERSATION_TURN.contains("Do not use emojis anywhere in the answer.")
    );
    assert!(CHAT_VIDEO_OBSERVATION_PROMPT.contains("Do not use emojis."));
}
