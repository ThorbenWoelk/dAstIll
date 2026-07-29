use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::models::{
    ContentItem, ContentItemKind, ContentPart, ContentPartKind, ContentSource, ContentSourceKind,
    ContentStatus, OpenAlexSavedSearchQuery, OpenAlexSearchScope, OpenAlexSort, ProviderIdentity,
    ProviderKind, SourceBackingKind, SubscriptionContainer, SubscriptionContainerKind,
};

use super::build_http_client;
use super::providers::{
    ProviderAdapterError, QuerySourceAdapter, ResolvedSourceDraft, SyncedSourceBatch,
};

fn slugify_query(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_dash = false;

    for ch in value.trim().chars() {
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

    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "query".to_string()
    } else {
        trimmed.to_string()
    }
}

fn saved_search_has_non_default_options(query: &OpenAlexSavedSearchQuery) -> bool {
    query
        .from_publication_date
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || query
            .to_publication_date
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || query
            .work_type
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || query.open_access_only == Some(true)
        || query.search_scope != OpenAlexSearchScope::default()
        || query.sort != OpenAlexSort::default()
}

fn saved_search_fingerprint(query: &OpenAlexSavedSearchQuery) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.query_text.trim().as_bytes());
    hasher.update([0]);
    hasher.update(
        query
            .from_publication_date
            .as_deref()
            .unwrap_or("")
            .trim()
            .as_bytes(),
    );
    hasher.update([0]);
    hasher.update(
        query
            .to_publication_date
            .as_deref()
            .unwrap_or("")
            .trim()
            .as_bytes(),
    );
    hasher.update([0]);
    hasher.update(query.work_type.as_deref().unwrap_or("").trim().as_bytes());
    hasher.update([0]);
    hasher.update(match query.open_access_only {
        Some(true) => b"oa:1".as_slice(),
        Some(false) => b"oa:0".as_slice(),
        None => b"oa:".as_slice(),
    });
    hasher.update([0]);
    hasher.update(match query.search_scope {
        OpenAlexSearchScope::GeneralSearch => b"scope:general".as_slice(),
        OpenAlexSearchScope::TitleAndAbstract => b"scope:title_abstract".as_slice(),
    });
    hasher.update([0]);
    hasher.update(match query.sort {
        OpenAlexSort::PublicationDateDesc => b"sort:pub_desc".as_slice(),
        OpenAlexSort::RelevanceScoreDesc => b"sort:rel_desc".as_slice(),
    });
    hasher
        .finalize()
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Stable saved-search id. Query-text slug alone ignores filters/sort/scope, so
/// distinct searches collapsed onto one shared catalog row and overwrote each other.
pub fn saved_search_query_id(query: &OpenAlexSavedSearchQuery) -> String {
    let slug = slugify_query(&query.query_text);
    if !saved_search_has_non_default_options(query) {
        return slug;
    }
    format!("{slug}:{}", saved_search_fingerprint(query))
}

fn sort_value(sort: OpenAlexSort) -> &'static str {
    match sort {
        OpenAlexSort::PublicationDateDesc => "publication_date:desc",
        OpenAlexSort::RelevanceScoreDesc => "relevance_score:desc",
    }
}

fn build_filter_clause(query: &OpenAlexSavedSearchQuery) -> String {
    let mut clauses = vec![
        "has_abstract:true".to_string(),
        "is_paratext:false".to_string(),
    ];
    if let Some(from_date) = query.from_publication_date.as_deref() {
        clauses.push(format!("from_publication_date:{from_date}"));
    }
    if let Some(to_date) = query.to_publication_date.as_deref() {
        clauses.push(format!("to_publication_date:{to_date}"));
    }
    if let Some(work_type) = query.work_type.as_deref() {
        clauses.push(format!("type:{work_type}"));
    }
    if query.open_access_only == Some(true) {
        clauses.push("is_oa:true".to_string());
    }
    clauses.join(",")
}

fn parse_openalex_date(value: &str) -> Option<DateTime<Utc>> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?;
    Some(DateTime::<Utc>::from_naive_utc_and_offset(
        date.and_hms_opt(0, 0, 0)?,
        Utc,
    ))
}

fn compact_openalex_id(id: &str) -> String {
    id.rsplit('/').next().unwrap_or(id).to_string()
}

fn reconstruct_abstract(inverted_index: &HashMap<String, Vec<usize>>) -> Option<String> {
    if inverted_index.is_empty() {
        return None;
    }

    let max_position = inverted_index
        .values()
        .flat_map(|positions| positions.iter().copied())
        .max()?;
    let mut tokens = vec![String::new(); max_position + 1];

    for (word, positions) in inverted_index {
        for position in positions {
            if *position < tokens.len() {
                tokens[*position] = word.clone();
            }
        }
    }

    let abstract_text = tokens
        .into_iter()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if abstract_text.is_empty() {
        None
    } else {
        Some(abstract_text)
    }
}

fn map_openalex_item(source: &ContentSource, work: &OpenAlexWork) -> ContentItem {
    let compact_id = compact_openalex_id(&work.id);
    let title = work
        .display_name
        .as_deref()
        .or(work.title.as_deref())
        .unwrap_or("Untitled work")
        .to_string();
    let mut external_ids = vec![ProviderIdentity {
        provider: ProviderKind::OpenAlex,
        external_id: work.id.clone(),
    }];
    if let Some(doi) = work.doi.as_deref() {
        external_ids.push(ProviderIdentity {
            provider: ProviderKind::OpenAlex,
            external_id: doi.to_string(),
        });
    }

    ContentItem {
        id: format!("openalex:work:{compact_id}"),
        source_id: source.id.clone(),
        provider: ProviderKind::OpenAlex,
        item_kind: ContentItemKind::Publication,
        title,
        thumbnail_url: None,
        published_at: work
            .publication_date
            .as_deref()
            .and_then(parse_openalex_date),
        external_ids,
    }
}

fn map_openalex_parts(source: &ContentSource, work: &OpenAlexWork) -> Vec<ContentPart> {
    let compact_id = compact_openalex_id(&work.id);
    let item_id = format!("openalex:work:{compact_id}");
    let mut parts = Vec::new();

    if reconstruct_abstract(&work.abstract_inverted_index).is_some() {
        parts.push(ContentPart {
            id: format!("openalex:abstract:{compact_id}"),
            source_id: source.id.clone(),
            item_id: item_id.clone(),
            provider: ProviderKind::OpenAlex,
            part_kind: ContentPartKind::Abstract,
            status: ContentStatus::Ready,
            text_available: true,
        });
    }

    parts
}

#[derive(Debug, Deserialize)]
struct OpenAlexListResponse {
    results: Vec<OpenAlexWork>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexWork {
    id: String,
    #[serde(default)]
    doi: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    publication_date: Option<String>,
    #[serde(default)]
    abstract_inverted_index: HashMap<String, Vec<usize>>,
}

#[derive(Clone)]
pub struct OpenAlexService {
    client: Client,
    api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OpenAlexPublicationMaterial {
    pub item: ContentItem,
    pub abstract_text: Option<String>,
    pub watch_url: String,
    pub description: Option<String>,
}

impl OpenAlexService {
    pub fn new() -> Self {
        Self::with_client(build_http_client())
    }

    pub fn with_client(client: Client) -> Self {
        let api_key = std::env::var("OPEN_ALEX_API_KEY")
            .or_else(|_| std::env::var("OPENALEX_API_KEY"))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        Self { client, api_key }
    }

    fn build_saved_search_container(title: &str, query_id: &str) -> SubscriptionContainer {
        SubscriptionContainer {
            id: format!("openalex:saved-search:{query_id}"),
            kind: SubscriptionContainerKind::SavedSearch,
            title: title.to_string(),
            provider: ProviderKind::OpenAlex,
            backing_kind: SourceBackingKind::Query,
            user_editable: true,
            source_ids: vec![format!("openalex:query:{query_id}")],
        }
    }

    fn build_saved_search_source(query: &OpenAlexSavedSearchQuery) -> ContentSource {
        let query_id = saved_search_query_id(query);
        let title = if query.query_text.trim().is_empty() {
            query.natural_language_query.trim()
        } else {
            query.query_text.trim()
        };
        let container = Self::build_saved_search_container(title, &query_id);

        ContentSource {
            id: format!("openalex:query:{query_id}"),
            provider: ProviderKind::OpenAlex,
            source_kind: ContentSourceKind::SavedSearch,
            container_id: container.id,
            container_kind: container.kind,
            backing_kind: SourceBackingKind::Query,
            title: if query.query_text.trim().is_empty() {
                query.natural_language_query.trim().to_string()
            } else {
                query.query_text.trim().to_string()
            },
            subtitle: Some(query.natural_language_query.trim().to_string()),
            handle: None,
            thumbnail_url: None,
            requires_auth: false,
            public_content_available: true,
            entitled_content_available: true,
            external_ids: vec![ProviderIdentity {
                provider: ProviderKind::OpenAlex,
                external_id: query.query_text.clone(),
            }],
        }
    }

    fn list_works<'a>(
        &'a self,
        query: &'a OpenAlexSavedSearchQuery,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<OpenAlexWork>, ProviderAdapterError>> + Send + 'a>>
    {
        Box::pin(async move {
            if query.query_text.trim().is_empty() {
                return Err(ProviderAdapterError::InvalidInput(
                    "openalex saved searches require a non-empty query".to_string(),
                ));
            }

            let mut request = self
                .client
                .get("https://api.openalex.org/works")
                .query(&[("per_page", "25"), ("sort", sort_value(query.sort))]);

            let filter_clause = build_filter_clause(query);
            request = match query.search_scope {
                OpenAlexSearchScope::GeneralSearch => {
                    let request = request.query(&[("search", query.query_text.as_str())]);
                    if filter_clause.is_empty() {
                        request
                    } else {
                        request.query(&[("filter", filter_clause.as_str())])
                    }
                }
                OpenAlexSearchScope::TitleAndAbstract => {
                    let scoped_filter = if filter_clause.is_empty() {
                        format!("title_and_abstract.search:{}", query.query_text)
                    } else {
                        format!(
                            "title_and_abstract.search:{},{}",
                            query.query_text, filter_clause
                        )
                    };
                    request.query(&[("filter", scoped_filter.as_str())])
                }
            };

            if let Some(api_key) = self.api_key.as_deref() {
                request = request.query(&[("api_key", api_key)]);
            }

            let payload = request
                .send()
                .await
                .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
                .error_for_status()
                .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?
                .json::<OpenAlexListResponse>()
                .await
                .map_err(|error| ProviderAdapterError::Upstream(error.to_string()))?;

            Ok(payload.results)
        })
    }
}

impl Default for OpenAlexService {
    fn default() -> Self {
        Self::new()
    }
}

impl QuerySourceAdapter for OpenAlexService {
    fn resolve_query_source<'a>(
        &'a self,
        query: &'a OpenAlexSavedSearchQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedSourceDraft, ProviderAdapterError>> + Send + 'a>>
    {
        Box::pin(async move {
            if query.query_text.trim().is_empty() {
                return Err(ProviderAdapterError::InvalidInput(
                    "openalex sources require a non-empty query text".to_string(),
                ));
            }

            let source = Self::build_saved_search_source(query);
            let query_id = saved_search_query_id(query);
            let title = source.title.as_str();

            Ok(ResolvedSourceDraft {
                container: Self::build_saved_search_container(title, &query_id),
                source,
            })
        })
    }
}

impl OpenAlexService {
    pub fn sync_query_source<'a>(
        &'a self,
        source: &'a ContentSource,
        query: &'a OpenAlexSavedSearchQuery,
    ) -> Pin<Box<dyn Future<Output = Result<SyncedSourceBatch, ProviderAdapterError>> + Send + 'a>>
    {
        Box::pin(async move {
            if source.provider != ProviderKind::OpenAlex
                || source.source_kind != ContentSourceKind::SavedSearch
            {
                return Err(ProviderAdapterError::UnsupportedSourceKind(
                    source.source_kind,
                ));
            }

            let works = self.list_works(query).await?;
            let items = works
                .iter()
                .map(|work| map_openalex_item(source, work))
                .collect();
            let parts = works
                .iter()
                .flat_map(|work| map_openalex_parts(source, work))
                .collect();

            Ok(SyncedSourceBatch {
                items,
                parts,
                media_assets: Vec::new(),
            })
        })
    }

    pub fn sync_query_source_materials<'a>(
        &'a self,
        source: &'a ContentSource,
        query: &'a OpenAlexSavedSearchQuery,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<OpenAlexPublicationMaterial>, ProviderAdapterError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if source.provider != ProviderKind::OpenAlex
                || source.source_kind != ContentSourceKind::SavedSearch
            {
                return Err(ProviderAdapterError::UnsupportedSourceKind(
                    source.source_kind,
                ));
            }

            let works = self.list_works(query).await?;
            Ok(works
                .iter()
                .map(|work| {
                    let abstract_text = reconstruct_abstract(&work.abstract_inverted_index);
                    let watch_url = work.doi.clone().unwrap_or_else(|| work.id.clone());
                    OpenAlexPublicationMaterial {
                        item: map_openalex_item(source, work),
                        description: abstract_text.clone(),
                        abstract_text,
                        watch_url,
                    }
                })
                .collect())
        })
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
