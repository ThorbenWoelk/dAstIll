use std::collections::HashSet;

pub const MAX_FTS_QUERY_TERMS: usize = 4;

const SEARCH_STOPWORDS: &[&str] = &[
    "a", "about", "an", "and", "best", "find", "for", "how", "in", "is", "me", "of", "on", "or",
    "show", "the", "to", "video", "videos", "what", "which",
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

pub fn build_fts_phrase_query(query: &str) -> Option<String> {
    let raw_tokens = tokenize_search_terms(query);
    if raw_tokens.len() < 2 {
        return None;
    }

    Some(format!("\"{}\"", raw_tokens.join(" ")))
}

pub fn build_fts_query(query: &str) -> String {
    meaningful_search_terms(query)
        .into_iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub fn normalize_search_text(query: &str) -> String {
    let normalized_tokens = tokenize_search_terms(query)
        .into_iter()
        .filter(|token| is_meaningful_search_term(token))
        .collect::<Vec<_>>();

    if normalized_tokens.is_empty() {
        query.trim().to_string()
    } else {
        normalized_tokens.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_fts_phrase_query, build_fts_query, meaningful_search_terms, normalize_search_text,
    };

    #[test]
    fn meaningful_search_terms_keeps_claude() {
        assert_eq!(meaningful_search_terms("claude"), vec!["claude"]);
    }

    #[test]
    fn build_fts_phrase_query_preserves_multi_term_phrase() {
        assert_eq!(
            build_fts_phrase_query("one good thing"),
            Some("\"one good thing\"".to_string())
        );
    }

    #[test]
    fn build_fts_phrase_query_ignores_single_term_queries() {
        assert_eq!(build_fts_phrase_query("anthropic"), None);
    }

    #[test]
    fn build_fts_query_drops_search_wrapper_words() {
        assert_eq!(
            build_fts_query("find videos about AI backlash"),
            "\"ai\" AND \"backlash\""
        );
    }

    #[test]
    fn normalize_search_text_strips_search_wrappers() {
        assert_eq!(
            normalize_search_text("show me videos about AI backlash"),
            "ai backlash"
        );
    }
}
