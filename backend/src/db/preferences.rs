use libsql::params;
use serde::{Deserialize, Serialize};

use crate::models::UserPreferences;

use super::{Store, StoreError};

const LEGACY_DOCUMENT_ID: &str = "user";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredUserPreferences {
    user_id: String,
    data: UserPreferences,
}

fn preferences_storage_key(user_id: &str) -> String {
    format!("user-preferences/{}.json", preferences_document_id(user_id))
}

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

pub async fn sql_preferences_count(store: &Store) -> Result<usize, StoreError> {
    let mut rows = store
        .turso
        .query("SELECT COUNT(*) FROM preferences", ())
        .await?;
    let Some(row) = rows.next().await? else {
        return Ok(0);
    };
    let count: i64 = row.get(0)?;
    Ok(count.max(0) as usize)
}

pub async fn snapshot_preferences_count(store: &Store) -> Result<usize, StoreError> {
    Ok(store.list_keys("user-preferences/").await?.len())
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
            params![doc_id.clone(), json],
        )
        .await?;
    store
        .put_json(
            &preferences_storage_key(user_id),
            &StoredUserPreferences {
                user_id: doc_id.clone(),
                data: normalized.clone(),
            },
        )
        .await?;
    store
        .record_libsql_snapshot_delta(vec![super::LibsqlSnapshotDeltaOperation::PutPreferences {
            user_id: doc_id,
            data: normalized,
        }])
        .await?;
    Ok(())
}

pub async fn bootstrap_sql_preferences_from_store(store: &Store) -> Result<usize, StoreError> {
    let records: Vec<StoredUserPreferences> = store.load_all("user-preferences/").await?;
    if records.is_empty() {
        return Ok(0);
    }

    for record in &records {
        let normalized = normalize_preferences(record.data.clone());
        let json = serde_json::to_string(&normalized)?;
        store
            .turso
            .execute(
                "INSERT INTO preferences (user_id, data) VALUES (?1, ?2) ON CONFLICT(user_id) DO UPDATE SET data = excluded.data",
                params![record.user_id.clone(), json],
            )
            .await?;
    }

    Ok(records.len())
}

pub async fn export_sql_preferences_to_store(store: &Store) -> Result<usize, StoreError> {
    let mut rows = store
        .turso
        .query("SELECT user_id, data FROM preferences", ())
        .await?;
    let mut exported = 0usize;
    let mut operations = Vec::new();

    while let Some(row) = rows.next().await? {
        let user_id: String = row.get(0)?;
        let json: String = row.get(1)?;
        let Ok(data) = serde_json::from_str::<UserPreferences>(&json) else {
            continue;
        };
        store
            .put_json(
                &preferences_storage_key(&user_id),
                &StoredUserPreferences {
                    user_id: user_id.clone(),
                    data: data.clone(),
                },
            )
            .await?;
        operations.push(super::LibsqlSnapshotDeltaOperation::PutPreferences {
            user_id: user_id.clone(),
            data,
        });
        exported += 1;
    }

    if !operations.is_empty() {
        store.record_libsql_snapshot_delta(operations).await?;
    }

    Ok(exported)
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
#[path = "preferences_tests.rs"]
mod preferences_tests;
