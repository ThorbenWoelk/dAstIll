use chrono::Utc;
use libsql::params;

use super::{StoredUserPreferences, normalize_preferences};
use crate::models::{UserPreferences, VocabularyReplacement};

#[test]
fn normalize_preferences_trims_and_drops_invalid_vocabulary_rules() {
    let now = Utc::now();
    let preferences = UserPreferences {
        channel_order: vec![],
        channel_sort_mode: "custom".to_string(),
        vocabulary_replacements: vec![
            VocabularyReplacement {
                from: "  Open A I ".to_string(),
                to: " OpenAI ".to_string(),
                added_at: now,
            },
            VocabularyReplacement {
                from: "Anthropic".to_string(),
                to: "Anthropic".to_string(),
                added_at: now,
            },
        ],
    };

    let normalized = normalize_preferences(preferences);

    assert_eq!(normalized.vocabulary_replacements.len(), 1);
    assert_eq!(normalized.vocabulary_replacements[0].from, "Open A I");
    assert_eq!(normalized.vocabulary_replacements[0].to, "OpenAI");
    assert_eq!(normalized.vocabulary_replacements[0].added_at, now);
}

#[tokio::test]
async fn reconcile_sql_preferences_with_records_updates_canonical_rows_and_prunes_stale_rows() {
    let store = crate::db::Store::for_test().await;
    let stale = UserPreferences {
        channel_order: vec!["stale".to_string()],
        channel_sort_mode: "custom".to_string(),
        vocabulary_replacements: vec![],
    };
    store
        .sql
        .execute(
            "INSERT INTO preferences (user_id, data) VALUES (?1, ?2)",
            params![
                "stale-user",
                serde_json::to_string(&stale).expect("serialize stale preferences")
            ],
        )
        .await
        .expect("insert stale preferences");

    let canonical = UserPreferences {
        channel_order: vec!["channel-1".to_string()],
        channel_sort_mode: "alpha".to_string(),
        vocabulary_replacements: vec![],
    };
    let (reconciled, pruned) = super::reconcile_sql_preferences_with_records(
        &store,
        vec![StoredUserPreferences {
            user_id: "user-1".to_string(),
            data: canonical.clone(),
        }],
    )
    .await
    .expect("reconcile preferences");

    assert_eq!(reconciled, 1);
    assert_eq!(pruned, 1);
    assert_eq!(
        super::get_user_preferences(&store, "user-1")
            .await
            .expect("load canonical preferences")
            .channel_sort_mode,
        "alpha"
    );
    let pruned = super::get_user_preferences(&store, "stale-user")
        .await
        .expect("load pruned preferences");
    assert!(pruned.channel_order.is_empty());
    assert_eq!(pruned.channel_sort_mode, "custom");
    assert!(pruned.vocabulary_replacements.is_empty());
}
