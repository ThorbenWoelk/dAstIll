use super::{
    OpenAlexListResponse, OpenAlexService, compact_openalex_id, reconstruct_abstract,
    saved_search_query_id, slugify_query,
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
fn saved_search_query_id_keeps_legacy_slug_for_default_options() {
    let query = OpenAlexSavedSearchQuery {
        natural_language_query: "Machine learning".to_string(),
        query_text: "machine learning".to_string(),
        from_publication_date: None,
        to_publication_date: None,
        work_type: None,
        open_access_only: None,
        search_scope: OpenAlexSearchScope::TitleAndAbstract,
        sort: OpenAlexSort::PublicationDateDesc,
    };
    assert_eq!(saved_search_query_id(&query), "machine-learning");
}

#[test]
fn saved_search_query_id_distinguishes_filters_and_scope() {
    let base = OpenAlexSavedSearchQuery {
        natural_language_query: "Machine learning".to_string(),
        query_text: "machine learning".to_string(),
        from_publication_date: None,
        to_publication_date: None,
        work_type: None,
        open_access_only: None,
        search_scope: OpenAlexSearchScope::TitleAndAbstract,
        sort: OpenAlexSort::PublicationDateDesc,
    };
    let open_access = OpenAlexSavedSearchQuery {
        open_access_only: Some(true),
        ..base.clone()
    };
    let dated = OpenAlexSavedSearchQuery {
        from_publication_date: Some("2024-01-01".to_string()),
        ..base.clone()
    };
    let general_scope = OpenAlexSavedSearchQuery {
        search_scope: OpenAlexSearchScope::GeneralSearch,
        ..base.clone()
    };

    let open_access_id = saved_search_query_id(&open_access);
    let dated_id = saved_search_query_id(&dated);
    let general_id = saved_search_query_id(&general_scope);
    let base_id = saved_search_query_id(&base);

    assert_eq!(base_id, "machine-learning");
    assert_ne!(open_access_id, base_id);
    assert_ne!(dated_id, base_id);
    assert_ne!(general_id, base_id);
    assert_ne!(open_access_id, dated_id);
    assert!(open_access_id.starts_with("machine-learning:"));
    assert_eq!(
        saved_search_query_id(&open_access),
        open_access_id,
        "fingerprint must be stable"
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
    assert_eq!(resolved.source.id, "openalex:query:multimodal-ai");
    assert_eq!(
        resolved.container.id,
        "openalex:saved-search:multimodal-ai"
    );
    assert_eq!(resolved.source.container_id, resolved.container.id);
}

#[tokio::test]
async fn resolve_query_source_namespaces_filtered_searches() {
    let service = OpenAlexService::new();
    let resolved = service
        .resolve_query_source(&OpenAlexSavedSearchQuery {
            natural_language_query: "OA multimodal".to_string(),
            query_text: "multimodal ai".to_string(),
            from_publication_date: None,
            to_publication_date: None,
            work_type: None,
            open_access_only: Some(true),
            search_scope: OpenAlexSearchScope::TitleAndAbstract,
            sort: OpenAlexSort::PublicationDateDesc,
        })
        .await
        .expect("source should resolve");

    assert!(
        resolved
            .source
            .id
            .starts_with("openalex:query:multimodal-ai:")
    );
    assert_ne!(resolved.source.id, "openalex:query:multimodal-ai");
    assert_eq!(resolved.source.container_id, resolved.container.id);
}

