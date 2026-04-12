use libsql::params;

use crate::models::UserPreferences;

use super::{Store, StoreError};

const LEGACY_DOCUMENT_ID: &str = "user";

fn preferences_document_id(user_id: &str) -> String {
    let trimmed = user_id.trim();
    if trimmed.is_empty() {
        LEGACY_DOCUMENT_ID.to_string()
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
    get_user_preferences(store, LEGACY_DOCUMENT_ID).await
}

pub async fn get_user_preferences(
    store: &Store,
    user_id: &str,
) -> Result<UserPreferences, StoreError> {
    let doc_id = preferences_document_id(user_id);
    let mut rows = store
        .turso
        .query(
            "SELECT data FROM preferences WHERE user_id = ?1",
            params![doc_id],
        )
        .await?;

    let prefs = if let Some(row) = rows.next().await? {
        let json: String = row.get(0)?;
        serde_json::from_str::<UserPreferences>(&json).ok()
    } else {
        None
    };
    Ok(normalize_preferences(prefs.unwrap_or_default()))
}

pub async fn save_preferences(
    store: &Store,
    preferences: &UserPreferences,
) -> Result<(), StoreError> {
    save_user_preferences(store, LEGACY_DOCUMENT_ID, preferences).await
}

pub async fn save_user_preferences(
    store: &Store,
    user_id: &str,
    preferences: &UserPreferences,
) -> Result<(), StoreError> {
    let normalized = normalize_preferences(preferences.clone());
    let json = serde_json::to_string(&normalized)?;
    let doc_id = preferences_document_id(user_id);
    store
        .turso
        .execute(
            "INSERT INTO preferences (user_id, data) VALUES (?1, ?2) ON CONFLICT(user_id) DO UPDATE SET data = excluded.data",
            params![doc_id, json],
        )
        .await?;
    Ok(())
}

pub async fn migrate_legacy_preferences(store: &Store, user_id: &str) -> Result<(), StoreError> {
    let user_doc_id = preferences_document_id(user_id);
    if user_doc_id == LEGACY_DOCUMENT_ID {
        return Ok(());
    }

    // Check if user already has preferences
    let mut rows = store
        .turso
        .query(
            "SELECT 1 FROM preferences WHERE user_id = ?1",
            params![user_doc_id.clone()],
        )
        .await?;
    if rows.next().await?.is_some() {
        return Ok(());
    }

    // Copy legacy preferences to user
    let mut legacy_rows = store
        .turso
        .query(
            "SELECT data FROM preferences WHERE user_id = ?1",
            params![LEGACY_DOCUMENT_ID],
        )
        .await?;

    if let Some(row) = legacy_rows.next().await? {
        let json: String = row.get(0)?;
        if let Ok(prefs) = serde_json::from_str::<UserPreferences>(&json) {
            let normalized = normalize_preferences(prefs);
            let normalized_json = serde_json::to_string(&normalized)?;
            store
                .turso
                .execute(
                    "INSERT INTO preferences (user_id, data) VALUES (?1, ?2) ON CONFLICT(user_id) DO UPDATE SET data = excluded.data",
                    params![user_doc_id, normalized_json],
                )
                .await?;
        }
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
