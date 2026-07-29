use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};

use crate::models::{
    ContentItem, ContentItemKind, ContentSource, ContentSourceKind, ProviderIdentity, ProviderKind,
    SourceBackingKind, SubscriptionContainer, SubscriptionContainerKind,
};

use super::build_http_client;
use super::providers::ProviderAdapterError;

fn first_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect::<String>()
}

fn slugify_url(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_dash = false;
    for ch in value.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            previous_was_dash = false;
            ch.to_ascii_lowercase()
        } else if previous_was_dash {
            continue;
        } else {
            previous_was_dash = true;
            '-'
        };
        slug.push(mapped);
    }
    slug.trim_matches('-').to_string()
}

fn short_url_fingerprint(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Stable catalog key for a website URL.
///
/// Slugifying alone collapses distinct URLs (`foo_bar` vs `foo-bar`, trailing
/// slash, etc.) onto one id and lets later subscribe/sync overwrite shared
/// channel, video, and transcript rows. Append a fingerprint of the exact URL.
pub fn website_url_identity(url: &str) -> String {
    format!("{}:{}", slugify_url(url), short_url_fingerprint(url))
}

fn selector(value: &str) -> Option<Selector> {
    Selector::parse(value).ok()
}

fn extract_title(document: &Html) -> Option<String> {
    let selector = selector("title")?;
    document
        .select(&selector)
        .next()
        .map(|node| node.text().collect::<String>().trim().to_string())
        .filter(|title| !title.is_empty())
}

fn extract_readable_text(document: &Html) -> String {
    for candidate in ["article", "main", "body"] {
        let Some(selector) = selector(candidate) else {
            continue;
        };
        if let Some(node) = document.select(&selector).next() {
            let text = node
                .text()
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            if !text.is_empty() {
                return text;
            }
        }
    }
    String::new()
}

#[derive(Clone)]
pub struct WebsiteService {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct WebsitePageMaterial {
    pub source: ContentSource,
    pub container: SubscriptionContainer,
    pub item: ContentItem,
    pub page_url: String,
    pub title: String,
    pub text_content: String,
    pub excerpt: Option<String>,
    pub published_at: chrono::DateTime<Utc>,
}

impl WebsiteService {
    pub fn new() -> Self {
        Self::with_client(build_http_client())
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    pub async fn resolve_page(
        &self,
        url: &str,
    ) -> Result<WebsitePageMaterial, ProviderAdapterError> {
        let parsed = reqwest::Url::parse(url.trim())
            .map_err(|error| ProviderAdapterError::InvalidInput(error.to_string()))?;
        let html = self
            .client
            .get(parsed.clone())
            .send()
            .await
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
            .error_for_status()
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
            .text()
            .await
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?;

        let document = Html::parse_document(&html);
        let title = extract_title(&document).unwrap_or_else(|| parsed.as_str().to_string());
        let text_content = extract_readable_text(&document);
        if text_content.trim().is_empty() {
            return Err(ProviderAdapterError::Upstream(
                "website extraction produced no readable text".to_string(),
            ));
        }

        let url_key = website_url_identity(parsed.as_str());
        let source_id = format!("website:{url_key}");
        let container = SubscriptionContainer {
            id: "websites".to_string(),
            kind: SubscriptionContainerKind::StandaloneTrackedSource,
            title: "Websites".to_string(),
            provider: ProviderKind::Website,
            backing_kind: SourceBackingKind::Manual,
            user_editable: true,
            source_ids: vec![source_id.clone()],
        };
        let source = ContentSource {
            id: source_id.clone(),
            provider: ProviderKind::Website,
            source_kind: ContentSourceKind::Website,
            container_id: container.id.clone(),
            container_kind: container.kind,
            backing_kind: SourceBackingKind::Manual,
            title: title.clone(),
            subtitle: Some(parsed.as_str().to_string()),
            handle: Some(parsed.as_str().to_string()),
            thumbnail_url: None,
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: true,
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::Website,
                external_id: parsed.as_str().to_string(),
            }],
        };
        let excerpt = Some(first_chars(&text_content, 220));
        let item = ContentItem {
            id: format!("website:item:{url_key}"),
            source_id,
            provider: ProviderKind::Website,
            item_kind: ContentItemKind::Webpage,
            title: title.clone(),
            thumbnail_url: None,
            published_at: Some(Utc::now()),
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::Website,
                external_id: parsed.as_str().to_string(),
            }],
        };

        Ok(WebsitePageMaterial {
            source,
            container,
            item,
            page_url: parsed.as_str().to_string(),
            title,
            text_content,
            excerpt,
            published_at: Utc::now(),
        })
    }
}

impl Default for WebsiteService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
