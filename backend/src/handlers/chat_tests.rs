use axum::http::StatusCode;
use chrono::{Duration, Utc};

use super::{
    active_reply_key, append_user_message_to_conversation, apply_manual_conversation_title,
    lookup_active_reply, mark_manual_title_on_create, rank_channel_suggestions,
    rank_video_suggestions, require_authenticated_persistent_chat, take_active_replies_for_scope,
};
use crate::handlers::validate_nonempty;
use crate::models::{
    Channel, ChatConversation, ChatMessage, ChatMessageStatus, ChatRole, ChatTitleStatus,
    ContentStatus, Video,
};
use crate::read_cache::SuggestedVideo;
use crate::security::{AccessContext, AccessRole, AuthState};
use crate::services::ActiveChatHandle;

fn sample_conversation(title: Option<&str>, title_status: ChatTitleStatus) -> ChatConversation {
    let created_at = Utc::now() - Duration::minutes(5);
    ChatConversation {
        id: "conv-123".to_string(),
        title: title.map(str::to_string),
        title_status,
        created_at,
        updated_at: created_at,
        messages: Vec::new(),
    }
}

fn sample_message(role: ChatRole, content: &str) -> ChatMessage {
    ChatMessage {
        id: format!("msg-{content}"),
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

#[test]
fn first_user_message_sets_provisional_title_and_generating_status() {
    let mut conversation = sample_conversation(None, ChatTitleStatus::Idle);
    let updated_at = Utc::now();

    let should_auto_name = append_user_message_to_conversation(
        &mut conversation,
        sample_message(ChatRole::User, "Find the best Rust video"),
        Some("Find the best Rust video".to_string()),
        updated_at,
    );

    assert!(should_auto_name);
    assert_eq!(
        conversation.title.as_deref(),
        Some("Find the best Rust video")
    );
    assert_eq!(conversation.title_status, ChatTitleStatus::Generating);
    assert_eq!(conversation.messages.len(), 1);
    assert_eq!(conversation.updated_at, updated_at);
}

#[test]
fn first_user_message_keeps_manual_title_without_triggering_auto_name() {
    let mut conversation = sample_conversation(Some("My chosen title"), ChatTitleStatus::Manual);
    let updated_at = Utc::now();

    let should_auto_name = append_user_message_to_conversation(
        &mut conversation,
        sample_message(ChatRole::User, "Summarize this channel"),
        Some("Summarize this channel".to_string()),
        updated_at,
    );

    assert!(!should_auto_name);
    assert_eq!(conversation.title.as_deref(), Some("My chosen title"));
    assert_eq!(conversation.title_status, ChatTitleStatus::Manual);
    assert_eq!(conversation.messages.len(), 1);
    assert_eq!(conversation.updated_at, updated_at);
}

#[test]
fn follow_up_user_message_does_not_retrigger_auto_naming() {
    let mut conversation = sample_conversation(Some("Existing title"), ChatTitleStatus::Ready);
    conversation
        .messages
        .push(sample_message(ChatRole::User, "First question"));
    conversation
        .messages
        .push(sample_message(ChatRole::Assistant, "First answer"));
    let updated_at = Utc::now();

    let should_auto_name = append_user_message_to_conversation(
        &mut conversation,
        sample_message(ChatRole::User, "Follow-up question"),
        Some("Follow-up question".to_string()),
        updated_at,
    );

    assert!(!should_auto_name);
    assert_eq!(conversation.title.as_deref(), Some("Existing title"));
    assert_eq!(conversation.title_status, ChatTitleStatus::Ready);
    assert_eq!(conversation.messages.len(), 3);
    assert_eq!(conversation.updated_at, updated_at);
}

#[test]
fn mark_manual_title_on_create_only_changes_conversations_with_titles() {
    let mut untitled = sample_conversation(None, ChatTitleStatus::Idle);
    mark_manual_title_on_create(&mut untitled);
    assert_eq!(untitled.title_status, ChatTitleStatus::Idle);

    let mut titled = sample_conversation(Some("Pinned title"), ChatTitleStatus::Idle);
    mark_manual_title_on_create(&mut titled);
    assert_eq!(titled.title_status, ChatTitleStatus::Manual);
}

#[test]
fn validate_nonempty_trims_and_rejects_blank_values() {
    assert_eq!(
        validate_nonempty("  Useful title  ", "must not be empty").unwrap(),
        "Useful title"
    );
    assert!(validate_nonempty("   ", "must not be empty").is_err());
}

#[test]
fn apply_manual_conversation_title_updates_state() {
    let mut conversation = sample_conversation(Some("Old"), ChatTitleStatus::Generating);
    let updated_at = Utc::now();

    apply_manual_conversation_title(&mut conversation, "New title", updated_at);

    assert_eq!(conversation.title.as_deref(), Some("New title"));
    assert_eq!(conversation.title_status, ChatTitleStatus::Manual);
    assert_eq!(conversation.updated_at, updated_at);
}

fn sample_channel(id: &str, name: &str, handle: Option<&str>) -> Channel {
    Channel {
        id: id.to_string(),
        handle: handle.map(str::to_string),
        name: name.to_string(),
        thumbnail_url: None,
        added_at: Utc::now(),
        earliest_sync_date: None,
        earliest_sync_date_user_set: false,
    }
}

#[test]
fn suggest_channels_prefers_handle_prefix_matches() {
    let channels = vec![
        sample_channel("chan-1", "HealthyGamerGG", Some("@healthygamergg")),
        sample_channel("chan-2", "Theo - t3.gg", Some("@t3dotgg")),
    ];

    let items = rank_channel_suggestions(&channels, "hea", 5);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].label, "HealthyGamerGG");
}

fn sample_video(id: &str, channel_id: &str, title: &str, age_days: i64) -> SuggestedVideo {
    let video = Video {
        id: id.to_string(),
        channel_id: channel_id.to_string(),
        title: title.to_string(),
        thumbnail_url: None,
        published_at: Utc::now() - Duration::days(age_days),
        is_short: false,
        transcript_status: ContentStatus::Ready,
        summary_status: ContentStatus::Ready,
        acknowledged: false,
        retry_count: 0,
        quality_score: None,
    };
    SuggestedVideo {
        id: video.id,
        channel_id: video.channel_id,
        title: video.title,
        published_at: video.published_at,
    }
}

#[test]
fn suggest_videos_prefers_newer_titles_on_ties() {
    let older = sample_video("vid-old", "chan-1", "Effort and Change", 10);
    let newer = sample_video("vid-new", "chan-1", "Effort and Change Again", 1);
    let channels = vec![sample_channel(
        "chan-1",
        "HealthyGamerGG",
        Some("@healthygamergg"),
    )];

    let items = rank_video_suggestions(&[older, newer], &channels, "eff", 5);

    assert_eq!(items[0].id, "vid-new");
}

fn auth_context(user_id: &str) -> AccessContext {
    AccessContext {
        user_id: Some(user_id.to_string()),
        auth_state: AuthState::Authenticated,
        access_role: AccessRole::User,
        allowed_channel_ids: Vec::new(),
        allowed_other_video_ids: Vec::new(),
    }
}

fn anonymous_context() -> AccessContext {
    AccessContext {
        user_id: None,
        auth_state: AuthState::Anonymous,
        access_role: AccessRole::Anonymous,
        allowed_channel_ids: Vec::new(),
        allowed_other_video_ids: Vec::new(),
    }
}

#[test]
fn active_reply_key_separates_anonymous_and_authenticated_scope_for_same_id() {
    let authenticated = auth_context("anonymous");
    let anonymous = anonymous_context();

    assert_ne!(
        active_reply_key(&authenticated, "conv-shared"),
        active_reply_key(&anonymous, "conv-shared")
    );
}

#[test]
fn persistent_chat_requires_authenticated_context() {
    let authenticated = auth_context("user-a");
    assert_eq!(
        require_authenticated_persistent_chat(&authenticated).unwrap(),
        "user-a"
    );

    let anonymous = anonymous_context();
    let error = require_authenticated_persistent_chat(&anonymous)
        .expect_err("anonymous access should be rejected");
    assert_eq!(error.0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn resume_and_cancel_require_matching_scope_ownership() {
    let owner = auth_context("user-a");
    let foreign = auth_context("user-b");
    let conversation_id = "conv-shared".to_string();
    let owner_key = active_reply_key(&owner, &conversation_id);
    let active_replies = tokio::sync::Mutex::new(std::collections::HashMap::from([(
        owner_key.clone(),
        ActiveChatHandle::new(),
    )]));

    let reconnect_error = lookup_active_reply(&active_replies, &foreign, &conversation_id)
        .await
        .expect_err("foreign reconnect should fail");
    assert_eq!(reconnect_error.0, StatusCode::NOT_FOUND);

    let _ = lookup_active_reply(&active_replies, &owner, &conversation_id)
        .await
        .expect("owner reconnect should succeed");

    let cancel_error = lookup_active_reply(&active_replies, &foreign, &conversation_id)
        .await
        .expect_err("foreign cancel should fail");
    assert_eq!(cancel_error.0, StatusCode::NOT_FOUND);
    assert!(active_replies.lock().await.contains_key(&owner_key));

    let active_reply = lookup_active_reply(&active_replies, &owner, &conversation_id)
        .await
        .expect("owner cancel should succeed");
    active_reply.cancel();
    assert!(active_replies.lock().await.contains_key(&owner_key));
}

#[test]
fn take_active_replies_for_scope_only_takes_matching_scope_entries() {
    let authenticated = auth_context("user-a");
    let anonymous = anonymous_context();
    let conversation_id = "conv-shared";

    let authenticated_key = active_reply_key(&authenticated, conversation_id);
    let anonymous_key = active_reply_key(&anonymous, conversation_id);
    let mut active_replies = std::collections::HashMap::new();
    active_replies.insert(authenticated_key.clone(), ActiveChatHandle::new());
    active_replies.insert(anonymous_key.clone(), ActiveChatHandle::new());

    let removed = take_active_replies_for_scope(&mut active_replies, "user:user-a");
    assert_eq!(removed.len(), 1);
    assert!(!active_replies.contains_key(&authenticated_key));
    assert!(active_replies.contains_key(&anonymous_key));
}
