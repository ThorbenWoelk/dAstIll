use super::{is_cloud_model, is_provider_capacity_limited_message};

#[test]
fn detects_cloud_models_with_colon_or_hyphen_suffixes() {
    assert!(is_cloud_model("minimax-m2.5:cloud"));
    assert!(is_cloud_model("qwen3.5:397b-cloud"));
    assert!(!is_cloud_model("qwen3:8b"));
}

#[test]
fn detects_provider_capacity_messages() {
    assert!(is_provider_capacity_limited_message(
        "HttpError: Invalid status code 429 Too Many Requests"
    ));
    assert!(is_provider_capacity_limited_message(
        r#"HttpError: Invalid status code 403 Forbidden with message: {"error":"this model requires a subscription, upgrade for access"}"#
    ));
    assert!(is_provider_capacity_limited_message(
        "subscription limit reached"
    ));
    assert!(is_provider_capacity_limited_message(
        "cloud cooldown active and no fallback model configured"
    ));
    assert!(is_provider_capacity_limited_message(
        "you have reached your weekly usage limit"
    ));
    assert!(!is_provider_capacity_limited_message(
        "HttpError: Invalid status code 403 Forbidden with message: unauthorized"
    ));
}
