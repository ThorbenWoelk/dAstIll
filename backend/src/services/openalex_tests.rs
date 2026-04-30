use super::{
    OpenAlexListResponse, OpenAlexService, compact_openalex_id, reconstruct_abstract, slugify_query,
};
use crate::models::{
    ContentSourceKind, OpenAlexSavedSearchQuery, OpenAlexSearchScope, OpenAlexSort, ProviderKind,
    SourceBackingKind, SubscriptionContainerKind,
};
use crate::services::QuerySourceAdapter;

#[test]
fn slugify_query_normalizes_saved_search_ids() {
    assert_eq!(
        slugify_query("Recent multimodal AI papers"),
        "recent-multimodal-ai-papers"
    );
}

#[test]
fn reconstruct_abstract_orders_words_by_position() {
    let payload: OpenAlexListResponse = serde_json::from_str(
        r#"{
          "results": [
            {
              "id": "https://openalex.org/W123",
              "display_name": "Example",
              "publication_date": "2025-01-02",
              "abstract_inverted_index": {
                "learning": [1],
                "machine": [0]
              }
            }
          ]
        }"#,
    )
    .expect("json should parse");

    let text = reconstruct_abstract(&payload.results[0].abstract_inverted_index)
        .expect("abstract should reconstruct");
    assert_eq!(text, "machine learning");
    assert_eq!(compact_openalex_id(&payload.results[0].id), "W123");
}

#[tokio::test]
async fn resolve_query_source_builds_saved_search_contract() {
    let service = OpenAlexService::new();
    let resolved = service
        .resolve_query_source(&OpenAlexSavedSearchQuery {
            natural_language_query: "Multimodal AI".to_string(),
            query_text: "multimodal ai".to_string(),
            from_publication_date: None,
            to_publication_date: None,
            work_type: None,
            open_access_only: None,
            search_scope: OpenAlexSearchScope::TitleAndAbstract,
            sort: OpenAlexSort::PublicationDateDesc,
        })
        .await
        .expect("source should resolve");

    assert_eq!(
        resolved.container.kind,
        SubscriptionContainerKind::SavedSearch
    );
    assert_eq!(resolved.container.provider, ProviderKind::OpenAlex);
    assert_eq!(resolved.source.source_kind, ContentSourceKind::SavedSearch);
    assert_eq!(resolved.source.backing_kind, SourceBackingKind::Query);
    assert_eq!(resolved.source.subtitle.as_deref(), Some("Multimodal AI"));
}
