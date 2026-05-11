use crate::models::{MediaAsset, MediaAssetKind};

use super::{Store, StoreError};

fn media_asset_kind_slug(asset_kind: MediaAssetKind) -> &'static str {
    match asset_kind {
        MediaAssetKind::SourceAudio => "source-audio",
        MediaAssetKind::GeneratedSummaryAudio => "generated-summary-audio",
    }
}

fn media_asset_key(item_id: &str, asset_kind: MediaAssetKind) -> String {
    format!(
        "media-assets/{item_id}/{}.json",
        media_asset_kind_slug(asset_kind)
    )
}

pub async fn upsert_media_asset(store: &Store, asset: &MediaAsset) -> Result<(), StoreError> {
    store
        .put_json(&media_asset_key(&asset.item_id, asset.asset_kind), asset)
        .await
}

pub async fn get_media_asset(
    store: &Store,
    item_id: &str,
    asset_kind: MediaAssetKind,
) -> Result<Option<MediaAsset>, StoreError> {
    store.get_json(&media_asset_key(item_id, asset_kind)).await
}

pub async fn get_source_audio_asset(
    store: &Store,
    item_id: &str,
) -> Result<Option<MediaAsset>, StoreError> {
    get_media_asset(store, item_id, MediaAssetKind::SourceAudio).await
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
