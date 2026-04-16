use crate::models::{
    ContentItem, ContentPart, ContentProvider, ContentSource, MediaAsset, WebsiteFolder,
};

/// Shared contract boundary for provider-specific discovery and ingestion work.
/// Concrete adapters are expected to keep provider rules behind this seam and
/// emit canonical library records.
pub trait ProviderAdapter {
    fn provider(&self) -> ContentProvider;
}

pub trait SourceDiscoveryAdapter: ProviderAdapter {
    fn list_subscribable_sources(&self) -> Vec<ContentSource>;
}

pub trait ItemSyncAdapter: ProviderAdapter {
    fn sync_source_items(&self, source: &ContentSource) -> Vec<ContentItem>;
}

pub trait PartIngestionAdapter: ProviderAdapter {
    fn extract_parts(&self, item: &ContentItem) -> Vec<ContentPart>;
    fn extract_media_assets(&self, item: &ContentItem) -> Vec<MediaAsset>;
}

pub trait WebsiteFolderAdapter: ProviderAdapter {
    fn list_folders(&self) -> Vec<WebsiteFolder>;
}
