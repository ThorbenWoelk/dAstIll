use super::{
    build_fts_phrase_queries, build_fts_query, build_fts_relaxed_query, meaningful_search_terms,
    normalize_search_text,
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
fn normalize_search_text_drops_creator_attribution_wrappers() {
    assert_eq!(
        normalize_search_text("the video where theo says anthropic is lying"),
        "theo anthropic lying"
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
