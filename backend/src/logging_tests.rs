use tracing::Level;

use super::{should_send_to_logfire, strip_wrapping_quotes};

#[test]
fn strips_debug_wrapped_quotes_from_messages() {
    assert_eq!(
        strip_wrapping_quotes("\"search indexing round complete\"".into()),
        "search indexing round complete"
    );
    assert_eq!(
        strip_wrapping_quotes("no wrapping quotes".into()),
        "no wrapping quotes"
    );
}

#[test]
fn sends_ai_targets_to_logfire() {
    assert!(should_send_to_logfire(
        "dastill::services::chat::reply",
        &Level::INFO
    ));
    assert!(should_send_to_logfire(
        "dastill::services::search",
        &Level::WARN
    ));
}

#[test]
fn sends_all_errors_to_logfire() {
    assert!(should_send_to_logfire(
        "dastill::handlers::videos",
        &Level::ERROR
    ));
    assert!(!should_send_to_logfire(
        "dastill::handlers::videos",
        &Level::INFO
    ));
}
