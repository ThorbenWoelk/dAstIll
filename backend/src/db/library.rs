use chrono::Utc;

use crate::models::WebsiteFolder;

use super::{Store, StoreError};

fn website_folder_prefix(user_id: &str) -> String {
    format!("users/{user_id}/website-folders/")
}

fn website_folder_key(user_id: &str, folder_id: &str) -> String {
    format!("{}{}.json", website_folder_prefix(user_id), folder_id)
}

fn next_website_folder_id() -> String {
    format!(
        "wf-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

async fn persist_website_folder(
    store: &Store,
    user_id: &str,
    folder: &WebsiteFolder,
) -> Result<(), StoreError> {
    store
        .put_json(&website_folder_key(user_id, &folder.id), folder)
        .await
}

pub async fn list_website_folders(
    store: &Store,
    user_id: &str,
) -> Result<Vec<WebsiteFolder>, StoreError> {
    let mut folders: Vec<WebsiteFolder> = store.load_all(&website_folder_prefix(user_id)).await?;
    folders.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(folders)
}

pub async fn get_website_folder(
    store: &Store,
    user_id: &str,
    folder_id: &str,
) -> Result<Option<WebsiteFolder>, StoreError> {
    store
        .get_json(&website_folder_key(user_id, folder_id))
        .await
}

pub async fn create_website_folder(
    store: &Store,
    user_id: &str,
    name: &str,
) -> Result<WebsiteFolder, StoreError> {
    let existing = list_website_folders(store, user_id).await?;
    let now = Utc::now();
    let folder = WebsiteFolder {
        id: next_website_folder_id(),
        name: name.trim().to_string(),
        position: existing.len(),
        created_at: now,
        updated_at: now,
        website_count: 0,
    };
    persist_website_folder(store, user_id, &folder).await?;
    Ok(folder)
}

pub async fn update_website_folder_name(
    store: &Store,
    user_id: &str,
    folder_id: &str,
    name: &str,
) -> Result<Option<WebsiteFolder>, StoreError> {
    let Some(mut folder) = get_website_folder(store, user_id, folder_id).await? else {
        return Ok(None);
    };
    folder.name = name.trim().to_string();
    folder.updated_at = Utc::now();
    persist_website_folder(store, user_id, &folder).await?;
    Ok(Some(folder))
}

pub async fn reorder_website_folders(
    store: &Store,
    user_id: &str,
    ordered_folder_ids: &[String],
) -> Result<Vec<WebsiteFolder>, StoreError> {
    let existing = list_website_folders(store, user_id).await?;
    let by_id = existing
        .into_iter()
        .map(|folder| (folder.id.clone(), folder))
        .collect::<std::collections::HashMap<_, _>>();

    let mut ordered = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for folder_id in ordered_folder_ids {
        if let Some(folder) = by_id.get(folder_id).cloned() {
            ordered.push(folder);
            seen.insert(folder_id.clone());
        }
    }

    let mut remaining = by_id
        .into_iter()
        .filter(|(folder_id, _)| !seen.contains(folder_id))
        .map(|(_, folder)| folder)
        .collect::<Vec<_>>();
    remaining.sort_by(|left, right| left.position.cmp(&right.position));
    ordered.extend(remaining);

    let now = Utc::now();
    for (position, folder) in ordered.iter_mut().enumerate() {
        folder.position = position;
        folder.updated_at = now;
        persist_website_folder(store, user_id, folder).await?;
    }

    Ok(ordered)
}

pub async fn delete_website_folder(
    store: &Store,
    user_id: &str,
    folder_id: &str,
) -> Result<bool, StoreError> {
    let key = website_folder_key(user_id, folder_id);
    let exists = store.key_exists(&key).await?;
    if !exists {
        return Ok(false);
    }

    store.delete_key(&key).await?;
    let remaining_ids = list_website_folders(store, user_id)
        .await?
        .into_iter()
        .map(|folder| folder.id)
        .collect::<Vec<_>>();
    let _ = reorder_website_folders(store, user_id, &remaining_ids).await?;
    Ok(true)
}
