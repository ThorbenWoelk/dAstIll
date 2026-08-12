use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::models::{
    ContentItem, ContentItemKind, ContentSource, ContentSourceKind, ProviderIdentity, ProviderKind,
    SourceBackingKind, SubscriptionContainer, SubscriptionContainerKind,
};

use super::build_http_client;
use super::providers::ProviderAdapterError;

/// Hard cap on HTML fetched during website subscribe/resolve.
///
/// Subscribe accepts attacker-controlled URLs. Buffering unbounded `.text()`
/// responses lets a single authenticated request OOM the Cloud Run instance.
pub(crate) const MAX_WEBSITE_HTML_BYTES: u64 = 5 * 1024 * 1024;

fn first_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect::<String>()
}

pub(crate) async fn read_response_text_limited(
    response: reqwest::Response,
    max_bytes: u64,
) -> Result<String, ProviderAdapterError> {
    if let Some(length) = response.content_length() {
        if length > max_bytes {
            return Err(ProviderAdapterError::Upstream(format!(
                "website response is too large: {length} bytes"
            )));
        }
    }

    let mut body = Vec::new();
    let mut response = response;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
    {
        let next_len = body.len().saturating_add(chunk.len());
        if next_len as u64 > max_bytes {
            return Err(ProviderAdapterError::Upstream(format!(
                "website response is too large: more than {max_bytes} bytes"
            )));
        }
        body.extend_from_slice(&chunk);
    }

    Ok(String::from_utf8_lossy(&body).into_owned())
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
        let response = self
            .client
            .get(parsed.clone())
            .send()
            .await
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
            .error_for_status()
            .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?;
        let html = read_response_text_limited(response, MAX_WEBSITE_HTML_BYTES).await?;

        let document = Html::parse_document(&html);
        let title = extract_title(&document).unwrap_or_else(|| parsed.as_str().to_string());
        let text_content = extract_readable_text(&document);
        if text_content.trim().is_empty() {
            return Err(ProviderAdapterError::Upstream(
                "website extraction produced no readable text".to_string(),
            ));
        }

        let source_id = format!("website:{}", slugify_url(parsed.as_str()));
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
            id: format!("website:item:{}", slugify_url(parsed.as_str())),
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
