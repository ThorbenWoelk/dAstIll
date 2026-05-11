use chrono::{Days, Utc};

use crate::models::{
    OpenAlexPlanResponse, OpenAlexSavedSearchQuery, OpenAlexSearchScope, OpenAlexSort,
};
use crate::services::ollama::CooldownStatusPolicy;

use super::{OllamaCore, OllamaPromptError};

fn parse_planner_json(response: &str) -> Option<OpenAlexSavedSearchQuery> {
    serde_json::from_str::<OpenAlexSavedSearchQuery>(response)
        .ok()
        .or_else(|| {
            let start = response.find('{')?;
            let end = response.rfind('}')?;
            serde_json::from_str::<OpenAlexSavedSearchQuery>(&response[start..=end]).ok()
        })
}

fn fallback_query_from_natural_language(natural_language_query: &str) -> OpenAlexSavedSearchQuery {
    let lowered = natural_language_query.to_ascii_lowercase();
    let recency_requested = ["recent", "latest", "new", "newest"]
        .iter()
        .any(|term| lowered.contains(term));
    let open_access_only = ["open access", "oa ", "free to read", "free papers"]
        .iter()
        .any(|term| lowered.contains(term));
    let work_type = if lowered.contains("review") {
        Some("review-article".to_string())
    } else if lowered.contains("preprint") {
        Some("preprint".to_string())
    } else if lowered.contains("paper") || lowered.contains("papers") {
        Some("article".to_string())
    } else {
        None
    };

    let query_text = lowered
        .replace("recent", "")
        .replace("latest", "")
        .replace("newest", "")
        .replace("new", "")
        .replace("papers", "")
        .replace("paper", "")
        .replace(" on ", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    let from_publication_date = if recency_requested {
        Some(
            Utc::now()
                .date_naive()
                .checked_sub_days(Days::new(365))
                .unwrap_or_else(|| Utc::now().date_naive())
                .format("%Y-%m-%d")
                .to_string(),
        )
    } else {
        None
    };

    OpenAlexSavedSearchQuery {
        natural_language_query: natural_language_query.to_string(),
        query_text: if query_text.is_empty() {
            natural_language_query.to_string()
        } else {
            query_text
        },
        from_publication_date,
        to_publication_date: None,
        work_type,
        open_access_only: open_access_only.then_some(true),
        search_scope: OpenAlexSearchScope::TitleAndAbstract,
        sort: if recency_requested {
            OpenAlexSort::PublicationDateDesc
        } else {
            OpenAlexSort::RelevanceScoreDesc
        },
    }
}

fn build_display_label(query: &OpenAlexSavedSearchQuery) -> String {
    if query.query_text.trim().is_empty() {
        "OpenAlex saved search".to_string()
    } else if query.from_publication_date.is_some() {
        format!("Recent {}", query.query_text.trim())
    } else {
        query.query_text.trim().to_string()
    }
}

fn build_notes(query: &OpenAlexSavedSearchQuery) -> Vec<String> {
    let mut notes = Vec::new();
    if let Some(from_date) = query.from_publication_date.as_deref() {
        notes.push(format!(
            "Results will be limited to works published on or after {from_date}."
        ));
    }
    if query.search_scope == OpenAlexSearchScope::TitleAndAbstract {
        notes.push("Matching uses title and abstract scope.".to_string());
    }
    if query.sort == OpenAlexSort::PublicationDateDesc {
        notes.push("Results will be sorted newest first.".to_string());
    }
    notes
}

const OPENALEX_PLANNER_PREAMBLE: &str = r#"You convert a natural-language literature subscription request into a structured OpenAlex saved-search query.

Return JSON only. No prose outside JSON.

Rules:
- `query_text` should be concise and keyword-oriented.
- If the user asks for "recent", "latest", or "new", infer a recent publication window.
- Prefer `title_and_abstract` scope for topic subscriptions unless the user clearly wants broad matching.
- Prefer `publication_date_desc` sort for recency-driven subscriptions.
- Use `relevance_score_desc` only when the request is primarily semantic/topic matching without recency intent.
- `work_type` should be a short OpenAlex-style type string like `article`, `preprint`, or `review-article` when clearly implied.
- `open_access_only` should be set only when the user explicitly asks for open access / free / OA.
- If you are unsure, keep optional fields null instead of inventing constraints.

Response shape:
{
  "natural_language_query": string,
  "query_text": string,
  "from_publication_date": string | null,
  "to_publication_date": string | null,
  "work_type": string | null,
  "open_access_only": boolean | null,
  "search_scope": "general_search" | "title_and_abstract",
  "sort": "publication_date_desc" | "relevance_score_desc"
}"#;

#[derive(Clone)]
pub struct OpenAlexPlannerService {
    core: OllamaCore,
}

impl OpenAlexPlannerService {
    pub fn new(core: OllamaCore) -> Self {
        Self { core }
    }

    pub async fn plan(&self, natural_language_query: &str) -> Result<OpenAlexPlanResponse, String> {
        let normalized = natural_language_query.trim();
        if normalized.is_empty() {
            return Err("OpenAlex planner requires a query".to_string());
        }

        let prompt =
            format!("Translate this request into an OpenAlex saved-search query:\n\n{normalized}");

        let query = match self
            .core
            .prompt_with_fallback(
                "openalex_query_planner",
                OPENALEX_PLANNER_PREAMBLE,
                &prompt,
                CooldownStatusPolicy::UseLocalFallback,
            )
            .await
        {
            Ok((response, _model_used)) => parse_planner_json(&response)
                .unwrap_or_else(|| fallback_query_from_natural_language(normalized)),
            Err(OllamaPromptError::NotAvailable)
            | Err(OllamaPromptError::RequestFailed(_))
            | Err(OllamaPromptError::GenerationFailed(_))
            | Err(OllamaPromptError::EmptyResponse)
            | Err(OllamaPromptError::InvalidStructuredResponse(_)) => {
                fallback_query_from_natural_language(normalized)
            }
        };

        Ok(OpenAlexPlanResponse {
            display_label: build_display_label(&query),
            notes: build_notes(&query),
            query,
        })
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
