use crate::models::UserPreferences;

use super::{Store, StoreError};

const COLLECTION: &str = "dastill_preferences";
const DOCUMENT_ID: &str = "user";

pub async fn get_preferences(store: &Store) -> Result<UserPreferences, StoreError> {
    let prefs: Option<UserPreferences> = store
        .firestore
        .fluent()
        .select()
        .by_id_in(COLLECTION)
        .obj()
        .one(DOCUMENT_ID)
        .await?;
    Ok(prefs.unwrap_or_default())
}

pub async fn save_preferences(
    store: &Store,
    preferences: &UserPreferences,
) -> Result<(), StoreError> {
    store
        .firestore
        .fluent()
        .update()
        .in_col(COLLECTION)
        .document_id(DOCUMENT_ID)
        .object(preferences)
        .execute::<UserPreferences>()
        .await?;
    Ok(())
}
