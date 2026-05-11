use super::*;

#[test]
fn every_entry_is_cloud_tagged() {
    use crate::services::http::is_cloud_model;
    for entry in CHAT_CLOUD_MODEL_CHOICES {
        assert!(
            is_cloud_model(entry.id),
            "expected cloud tag, got {}",
            entry.id
        );
    }
}

#[test]
fn default_prefers_configured_when_allowed() {
    assert_eq!(
        default_chat_cloud_model_id("glm-5.1:cloud"),
        "glm-5.1:cloud"
    );
    assert_eq!(
        default_chat_cloud_model_id("qwen3:8b"),
        CHAT_CLOUD_MODEL_CHOICES[0].id
    );
}
