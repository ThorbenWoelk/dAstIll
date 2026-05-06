use std::collections::HashSet;

pub const MAX_FTS_QUERY_TERMS: usize = 4;

const SEARCH_LEADING_WRAPPER_SEQUENCES: &[&[&str]] = &[
    &["where", "they", "talk", "about"],
    &["where", "they", "talk"],
    &["where", "they", "discuss"],
    &["where", "they", "mention"],
    &["they", "talk", "about"],
    &["they", "talk"],
    &["they", "discuss"],
    &["they", "mention"],
    &["talk", "about"],
];
const SEARCH_LEADING_WRAPPER_TOKENS: &[&str] = &[
    "about", "episode", "episodes", "find", "me", "on", "show", "video", "videos",
];
const SEARCH_STOPWORDS: &[&str] = &[
    "a", "about", "an", "and", "best", "episode", "episodes", "find", "for", "he", "how", "in",
    "is", "me", "of", "on", "or", "say", "says", "she", "show", "talk", "talked", "talks", "that",
    "the", "this", "to", "video", "videos", "what", "which", "where", "with",
];
const SHORT_TECHNICAL_SEARCH_TERMS: &[&str] = &["ai", "db", "go", "js", "ml", "ui", "ux"];

fn is_search_term_character(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-' | '.')
}

pub fn tokenize_search_terms(query: &str) -> Vec<String> {
    query
        .split(|character: char| !is_search_term_character(character))
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

fn strip_leading_search_wrapper_tokens(mut tokens: Vec<String>) -> Vec<String> {
    let original_tokens = tokens.clone();

    loop {
        let mut changed = false;

        for sequence in SEARCH_LEADING_WRAPPER_SEQUENCES {
            if tokens.len() >= sequence.len()
                && tokens
                    .iter()
                    .take(sequence.len())
                    .map(String::as_str)
                    .eq(sequence.iter().copied())
            {
                tokens.drain(..sequence.len());
                changed = true;
                break;
            }
        }

        if changed {
            continue;
        }

        if tokens
            .first()
            .is_some_and(|token| SEARCH_LEADING_WRAPPER_TOKENS.contains(&token.as_str()))
        {
            tokens.remove(0);
            continue;
        }

        break;
    }

    if tokens.is_empty() {
        original_tokens
    } else {
        tokens
    }
}

fn normalized_search_tokens(query: &str) -> Vec<String> {
    strip_leading_search_wrapper_tokens(tokenize_search_terms(query))
}

pub fn meaningful_search_terms(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();

    normalized_search_tokens(query)
        .into_iter()
        .filter(|token| is_meaningful_search_term(token))
        .filter(|token| seen.insert(token.clone()))
        .take(MAX_FTS_QUERY_TERMS)
        .collect()
}

fn build_multi_token_phrase_query(tokens: &[String]) -> Option<String> {
    if tokens.len() < 2 {
        return None;
    }

    Some(format!("\"{}\"", tokens.join(" ")))
}

pub fn normalize_search_text(query: &str) -> String {
    let normalized_tokens = normalized_search_tokens(query)
        .into_iter()
        .filter(|token| is_meaningful_search_term(token))
        .collect::<Vec<_>>();

    if normalized_tokens.is_empty() {
        query.trim().to_string()
    } else {
        normalized_tokens.join(" ")
    }
}

pub fn build_fts_phrase_queries(query: &str) -> Vec<String> {
    let mut phrases = Vec::new();

    let normalized_query = normalize_search_text(query);
    let normalized_tokens = tokenize_search_terms(&normalized_query);
    if let Some(normalized_phrase) = build_multi_token_phrase_query(&normalized_tokens) {
        phrases.push(normalized_phrase);
    }

    let raw_tokens = tokenize_search_terms(query);
    if let Some(raw_phrase) = build_multi_token_phrase_query(&raw_tokens) {
        if !phrases.iter().any(|phrase| phrase == &raw_phrase) {
            phrases.push(raw_phrase);
        }
    }

    phrases
}

pub fn build_fts_query(query: &str) -> String {
    meaningful_search_terms(query)
        .into_iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub fn build_fts_relaxed_query(query: &str) -> String {
    let terms = meaningful_search_terms(query);
    if terms.len() < 2 {
        return String::new();
    }

    terms
        .into_iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
#[path = "search_query_tests.rs"]
mod search_query_tests;
