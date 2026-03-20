use std::collections::HashSet;

pub const MAX_FTS_QUERY_TERMS: usize = 4;

const SEARCH_STOPWORDS: &[&str] = &[
    "a", "an", "and", "best", "for", "how", "in", "is", "of", "on", "or", "the", "to", "what",
    "which",
];
const SHORT_TECHNICAL_SEARCH_TERMS: &[&str] = &["ai", "db", "go", "js", "ml", "ui", "ux"];

pub fn tokenize_search_terms(query: &str) -> Vec<String> {
    query
        .split(|character: char| {
            !(character.is_alphanumeric() || matches!(character, '_' | '-' | '.'))
        })
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

pub fn is_meaningful_search_term(token: &str) -> bool {
    if token.len() < 2 || SEARCH_STOPWORDS.contains(&token) {
        return false;
    }

    token.len() >= 3 || SHORT_TECHNICAL_SEARCH_TERMS.contains(&token)
}

pub fn meaningful_search_terms(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();

    tokenize_search_terms(query)
        .into_iter()
        .filter(|token| is_meaningful_search_term(token))
        .filter(|token| seen.insert(token.clone()))
        .take(MAX_FTS_QUERY_TERMS)
        .collect()
}

pub fn build_fts_query(query: &str) -> String {
    meaningful_search_terms(query)
        .into_iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" AND ")
}
