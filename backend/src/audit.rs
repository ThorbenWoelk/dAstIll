//! Structured audit logs for security- and support-relevant mutations.
//! Filter production logs with tracing target `dastill_audit`.

use chrono::{DateTime, Utc};

use crate::models::{Channel, HighlightSource, UpdateChannelRequest, UserPreferences};

pub const TARGET: &str = "dastill_audit";

pub fn opt_dt(dt: Option<DateTime<Utc>>) -> String {
    dt.map(|d| d.to_rfc3339())
        .unwrap_or_else(|| "none".to_string())
}

pub fn log_channel_update(
    user_id: &str,
    channel_id: &str,
    before: &Channel,
    after: &Channel,
    payload: &UpdateChannelRequest,
) {
    tracing::info!(
        target: TARGET,
        event = "channel.update",
        user_id = %user_id,
        channel_id = %channel_id,
        old_earliest_sync_date = %opt_dt(before.earliest_sync_date),
        new_earliest_sync_date = %opt_dt(after.earliest_sync_date),
        old_earliest_sync_date_user_set = before.earliest_sync_date_user_set,
        new_earliest_sync_date_user_set = after.earliest_sync_date_user_set,
        requested_earliest_sync_date = ?payload.earliest_sync_date.map(|d| d.to_rfc3339()),
        requested_earliest_sync_date_user_set = ?payload.earliest_sync_date_user_set,
        "channel sync settings updated"
    );
}

pub fn log_channel_subscribe(user_id: &str, channel: &Channel, subscribe_input_len: usize) {
    tracing::info!(
        target: TARGET,
        event = "channel.subscribe",
        user_id = %user_id,
        channel_id = %channel.id,
        channel_name = %channel.name,
        initial_earliest_sync_date = %opt_dt(channel.earliest_sync_date),
        earliest_sync_date_user_set = channel.earliest_sync_date_user_set,
        subscribe_input_len = subscribe_input_len,
        "channel subscribed"
    );
}

pub fn log_channel_unsubscribe(user_id: &str, channel_id: &str) {
    tracing::info!(
        target: TARGET,
        event = "channel.unsubscribe",
        user_id = %user_id,
        channel_id = %channel_id,
        "channel subscription removed"
    );
}

pub fn log_video_acknowledgment(
    user_id: &str,
    video_id: &str,
    channel_id: &str,
    old_acknowledged: bool,
    new_acknowledged: bool,
) {
    tracing::info!(
        target: TARGET,
        event = "video.acknowledgment",
        user_id = %user_id,
        video_id = %video_id,
        channel_id = %channel_id,
        old_acknowledged = old_acknowledged,
        new_acknowledged = new_acknowledged,
        "video acknowledgment updated"
    );
}

pub fn log_manual_video_add(
    user_id: &str,
    video_id: &str,
    channel_id: &str,
    target_channel_id: &str,
    already_exists: bool,
) {
    tracing::info!(
        target: TARGET,
        event = "video.add_manual",
        user_id = %user_id,
        video_id = %video_id,
        channel_id = %channel_id,
        target_channel_id = %target_channel_id,
        already_exists = already_exists,
        "manual video added or linked"
    );
}

pub fn log_preferences_save(user_id: &str, before: &UserPreferences, after: &UserPreferences) {
    tracing::info!(
        target: TARGET,
        event = "preferences.save",
        user_id = %user_id,
        old_channel_sort_mode = %before.channel_sort_mode,
        new_channel_sort_mode = %after.channel_sort_mode,
        old_channel_order_len = before.channel_order.len(),
        new_channel_order_len = after.channel_order.len(),
        old_vocabulary_replacements_len = before.vocabulary_replacements.len(),
        new_vocabulary_replacements_len = after.vocabulary_replacements.len(),
        "user preferences saved"
    );
}

pub fn log_highlight_create(
    user_id: &str,
    video_id: &str,
    highlight_id: i64,
    source: HighlightSource,
) {
    tracing::info!(
        target: TARGET,
        event = "highlight.create",
        user_id = %user_id,
        video_id = %video_id,
        highlight_id = highlight_id,
        source = ?source,
        "highlight created"
    );
}

pub fn log_highlight_delete(user_id: &str, highlight_id: i64) {
    tracing::info!(
        target: TARGET,
        event = "highlight.delete",
        user_id = %user_id,
        highlight_id = highlight_id,
        "highlight deleted"
    );
}

pub fn log_chat_conversation_create(audit_scope: &str, conversation_id: &str, title_len: usize) {
    tracing::info!(
        target: TARGET,
        event = "chat.conversation.create",
        audit_scope = %audit_scope,
        conversation_id = %conversation_id,
        title_len = title_len,
        "chat conversation created"
    );
}

pub fn log_chat_conversation_update(
    audit_scope: &str,
    conversation_id: &str,
    old_title_len: usize,
    new_title_len: usize,
) {
    tracing::info!(
        target: TARGET,
        event = "chat.conversation.update",
        audit_scope = %audit_scope,
        conversation_id = %conversation_id,
        old_title_len = old_title_len,
        new_title_len = new_title_len,
        "chat conversation title updated"
    );
}

pub fn log_chat_conversation_delete(audit_scope: &str, conversation_id: &str) {
    tracing::info!(
        target: TARGET,
        event = "chat.conversation.delete",
        audit_scope = %audit_scope,
        conversation_id = %conversation_id,
        "chat conversation deleted"
    );
}

pub fn log_chat_conversations_delete_all(audit_scope: &str) {
    tracing::info!(
        target: TARGET,
        event = "chat.conversation.delete_all",
        audit_scope = %audit_scope,
        "all chat conversations deleted for scope"
    );
}

pub fn log_video_reset(video_id: &str, channel_id: &str) {
    tracing::info!(
        target: TARGET,
        event = "video.reset",
        video_id = %video_id,
        channel_id = %channel_id,
        "video transcript and summary reset"
    );
}
