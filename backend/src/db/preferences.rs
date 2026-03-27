use crate::models::UserPreferences;

use super::{Store, StoreError};

const COLLECTION: &str = "dastill_preferences";
const DOCUMENT_ID: &str = "user";

fn normalize_preferences(mut preferences: UserPreferences) -> UserPreferences {
    preferences.vocabulary_replacements = preferences
        .vocabulary_replacements
        .into_iter()
        .filter_map(|replacement| {
            let from = replacement.from.trim();
            let to = replacement.to.trim();
            if from.is_empty() || to.is_empty() || from == to {
                return None;
            }
            Some(crate::models::VocabularyReplacement {
                from: from.to_string(),
                to: to.to_string(),
                added_at: replacement.added_at,
            })
        })
        .collect();
    preferences
}

pub async fn get_preferences(store: &Store) -> Result<UserPreferences, StoreError> {
    let prefs: Option<UserPreferences> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(DOCUMENT_ID)
        .await?;
    Ok(normalize_preferences(prefs.unwrap_or_default()))
}

pub async fn save_preferences(
    store: &Store,
    preferences: &UserPreferences,
) -> Result<(), StoreError> {
    let normalized = normalize_preferences(preferences.clone());
    store
        .firestore
        .fluent()
        .update()
        .in_col(COLLECTION)
        .document_id(DOCUMENT_ID)
        .object(&normalized)
        .execute::<UserPreferences>()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
}
