use super::{is_explicit_realtime_status_query, is_recent_activity_query};

#[test]
fn detects_recent_activity_queries() {
    assert!(is_recent_activity_query(
        "What is HealthyGamerGG doing lately?"
    ));
    assert!(is_recent_activity_query(
        "What has Theo been talking about recently?"
    ));
    assert!(is_recent_activity_query(
        "What is HealthyGamerGG focused on these days?"
    ));
}

#[test]
fn detects_explicit_realtime_status_queries() {
    assert!(is_explicit_realtime_status_query(
        "Is HealthyGamerGG live right now?"
    ));
    assert!(is_explicit_realtime_status_query(
        "What is HealthyGamerGG working on this week outside YouTube?"
    ));
    assert!(!is_explicit_realtime_status_query(
        "What is HealthyGamerGG doing lately?"
    ));
}
