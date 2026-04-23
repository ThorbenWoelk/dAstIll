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
    "a", "about", "an", "and", "best", "episode", "episodes", "find", "for", "how", "in", "is",
    "me", "of", "on", "or", "show", "that", "the", "this", "to", "video", "videos", "what",
    "which", "with",
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

pub fn build_fts_phrase_queries(query: &str) -> Vec<String> {
    let mut phrases = Vec::new();

    let normalized_query = normalize_search_text(query);
    let normalized_tokens = tokenize_search_terms(&normalized_query);
    if normalized_tokens.len() >= 2 {
        phrases.push(format!("\"{}\"", normalized_tokens.join(" ")));
    }

    let raw_tokens = tokenize_search_terms(query);
    if raw_tokens.len() >= 2 {
        let raw_phrase = format!("\"{}\"", raw_tokens.join(" "));
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

#[cfg(test)]
mod tests {
    use super::{
        build_fts_phrase_queries, build_fts_query, build_fts_relaxed_query,
        meaningful_search_terms, normalize_search_text,
    };

    #[test]
    fn meaningful_search_terms_keeps_claude() {
        assert_eq!(meaningful_search_terms("claude"), vec!["claude"]);
    }

    #[test]
    fn build_fts_phrase_queries_preserve_multi_term_phrase() {
        assert_eq!(
            build_fts_phrase_queries("one good thing"),
            vec!["\"one good thing\"".to_string()]
        );
    }

    #[test]
    fn build_fts_phrase_queries_ignore_single_term_queries() {
        assert_eq!(build_fts_phrase_queries("anthropic"), Vec::<String>::new());
    }

    #[test]
    fn build_fts_query_drops_search_wrapper_words() {
        assert_eq!(
            build_fts_query("find videos about AI backlash"),
            "\"ai\" AND \"backlash\""
        );
    }

    #[test]
    fn build_fts_relaxed_query_broadens_multi_term_searches() {
        assert_eq!(
            build_fts_relaxed_query("open source is dead now"),
            "\"open\" OR \"source\" OR \"dead\" OR \"now\""
        );
    }

    #[test]
    fn normalize_search_text_strips_search_wrappers() {
        assert_eq!(
            normalize_search_text("show me videos about AI backlash"),
            "ai backlash"
        );
    }

    #[test]
    fn normalize_search_text_strips_conversational_wrapper_prefixes() {
        assert_eq!(
            normalize_search_text("video where they talk about one good thing"),
            "one good thing"
        );
    }

    #[test]
    fn normalize_search_text_drops_series_wrapper_words_inside_query() {
        assert_eq!(
            normalize_search_text("that hard fork episode with one good thing"),
            "hard fork one good thing"
        );
    }

    #[test]
    fn build_fts_phrase_queries_try_normalized_phrase_before_raw_wrapper_phrase() {
        assert_eq!(
            build_fts_phrase_queries("video where they talk about one good thing"),
            vec![
                "\"one good thing\"".to_string(),
                "\"video where they talk about one good thing\"".to_string(),
            ]
        );
    }

    #[test]
    fn meaningful_search_terms_drop_wrapper_prefixes_before_term_cap() {
        assert_eq!(
            meaningful_search_terms("video where they talk about one good thing"),
            vec!["one", "good", "thing"]
        );
    }
}
