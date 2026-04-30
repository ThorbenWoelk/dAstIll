use super::{build_display_label, fallback_query_from_natural_language, parse_planner_json};
use crate::models::{OpenAlexSearchScope, OpenAlexSort};

#[test]
fn fallback_recent_query_adds_date_and_newest_sort() {
    let query = fallback_query_from_natural_language("recent ai papers");
    assert!(query.from_publication_date.is_some());
    assert_eq!(query.search_scope, OpenAlexSearchScope::TitleAndAbstract);
    assert_eq!(query.sort, OpenAlexSort::PublicationDateDesc);
}

#[test]
fn parser_extracts_embedded_json() {
    let parsed = parse_planner_json(
        "Here you go {\"natural_language_query\":\"recent ai papers\",\"query_text\":\"artificial intelligence\",\"from_publication_date\":\"2025-01-01\",\"to_publication_date\":null,\"work_type\":\"article\",\"open_access_only\":null,\"search_scope\":\"title_and_abstract\",\"sort\":\"publication_date_desc\"}",
    )
    .expect("planner json should parse");

    assert_eq!(parsed.query_text, "artificial intelligence");
}

#[test]
fn display_label_prefers_recent_prefix() {
    let query = fallback_query_from_natural_language("recent ai papers");
    assert!(build_display_label(&query).starts_with("Recent "));
}
