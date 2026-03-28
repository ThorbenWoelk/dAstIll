use crate::models::UserPreferences;

use super::{Store, StoreError};

const COLLECTION: &str = "dastill_preferences";
const DOCUMENT_ID: &str = "user";

fn preferences_document_id(user_id: &str) -> String {
    let trimmed = user_id.trim();
    if trimmed.is_empty() {
        DOCUMENT_ID.to_string()
    } else {
        trimmed.to_string()
    }
}

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
    get_user_preferences(store, DOCUMENT_ID).await
}

pub async fn get_user_preferences(
    store: &Store,
    user_id: &str,
) -> Result<UserPreferences, StoreError> {
    let prefs: Option<UserPreferences> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(&preferences_document_id(user_id))
        .await?;
    Ok(normalize_preferences(prefs.unwrap_or_default()))
}

pub async fn save_preferences(
    store: &Store,
    preferences: &UserPreferences,
) -> Result<(), StoreError> {
    save_user_preferences(store, DOCUMENT_ID, preferences).await
}

pub async fn save_user_preferences(
    store: &Store,
    user_id: &str,
    preferences: &UserPreferences,
) -> Result<(), StoreError> {
    let normalized = normalize_preferences(preferences.clone());
    store
        .firestore
        .fluent()
        .update()
        .in_col(COLLECTION)
        .document_id(&preferences_document_id(user_id))
        .object(&normalized)
        .execute::<UserPreferences>()
        .await?;
    Ok(())
}

pub async fn migrate_legacy_preferences(
    store: &Store,
    user_id: &str,
) -> Result<(), StoreError> {
    let user_doc_id = preferences_document_id(user_id);
    if user_doc_id == DOCUMENT_ID {
        return Ok(());
    }

    let user_exists: bool = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj::<UserPreferences>()
        .one(&user_doc_id)
        .await?
        .is_some();

    if user_exists {
        return Ok(());
    }

    let legacy_prefs: Option<UserPreferences> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(DOCUMENT_ID)
        .await?;

    if let Some(prefs) = legacy_prefs {
        let normalized = normalize_preferences(prefs);
        store
            .firestore
            .fluent()
            .update()
            .in_col(COLLECTION)
            .document_id(&user_doc_id)
            .object(&normalized)
            .execute::<UserPreferences>()
            .await?;
    }

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
