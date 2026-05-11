use chrono::Utc;

use super::normalize_preferences;
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
