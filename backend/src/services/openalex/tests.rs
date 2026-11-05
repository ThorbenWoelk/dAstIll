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
fn saved_search_query_id_distinguishes_lossy_query_text_slugs() {
    let machine_hyphen = OpenAlexSavedSearchQuery {
        natural_language_query: "Machine learning".to_string(),
        query_text: "machine-learning".to_string(),
        from_publication_date: None,
        to_publication_date: None,
        work_type: None,
        open_access_only: None,
        search_scope: OpenAlexSearchScope::TitleAndAbstract,
        sort: OpenAlexSort::PublicationDateDesc,
    };
    let machine_underscore = OpenAlexSavedSearchQuery {
        query_text: "machine_learning".to_string(),
        ..machine_hyphen.clone()
    };
    let cpp = OpenAlexSavedSearchQuery {
        natural_language_query: "C++".to_string(),
        query_text: "C++".to_string(),
        from_publication_date: None,
        to_publication_date: None,
        work_type: None,
        open_access_only: None,
        search_scope: OpenAlexSearchScope::TitleAndAbstract,
        sort: OpenAlexSort::PublicationDateDesc,
    };
    let csharp = OpenAlexSavedSearchQuery {
        natural_language_query: "C#".to_string(),
        query_text: "C#".to_string(),
        ..cpp.clone()
    };

    let hyphen_id = saved_search_query_id(&machine_hyphen);
    let underscore_id = saved_search_query_id(&machine_underscore);
    let cpp_id = saved_search_query_id(&cpp);
    let csharp_id = saved_search_query_id(&csharp);

    assert_ne!(hyphen_id, underscore_id);
    assert!(hyphen_id.starts_with("machine-learning:"));
    assert!(underscore_id.starts_with("machine-learning:"));
    assert_ne!(cpp_id, csharp_id);
    assert!(cpp_id.starts_with("c:"));
    assert!(csharp_id.starts_with("c:"));
    assert_eq!(
        saved_search_query_id(&machine_hyphen),
        hyphen_id,
        "fingerprint must be stable"
    );
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

    let base_id = saved_search_query_id(&base);
    let open_access_id = saved_search_query_id(&open_access);
    let dated_id = saved_search_query_id(&dated);
    let general_id = saved_search_query_id(&general_scope);

    assert!(base_id.starts_with("machine-learning:"));
    assert_ne!(open_access_id, base_id);
    assert_ne!(dated_id, base_id);
    assert_ne!(general_id, base_id);
    assert_ne!(open_access_id, dated_id);
    assert!(open_access_id.starts_with("machine-learning:"));
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
    assert!(
        resolved
            .source
            .id
            .starts_with("openalex:query:multimodal-ai:")
    );
    assert_ne!(resolved.source.id, "openalex:query:multimodal-ai");
    assert_eq!(
        resolved.container.id,
        format!(
            "openalex:saved-search:{}",
            resolved.source.id.trim_start_matches("openalex:query:")
        )
    );
    assert_eq!(resolved.source.container_id, resolved.container.id);
}

#[tokio::test]
async fn resolve_query_source_namespaces_colliding_query_text() {
    let service = OpenAlexService::new();
    let first = service
        .resolve_query_source(&OpenAlexSavedSearchQuery {
            natural_language_query: "C++".to_string(),
            query_text: "C++".to_string(),
            from_publication_date: None,
            to_publication_date: None,
            work_type: None,
            open_access_only: None,
            search_scope: OpenAlexSearchScope::TitleAndAbstract,
            sort: OpenAlexSort::PublicationDateDesc,
        })
        .await
        .expect("source should resolve");
    let second = service
        .resolve_query_source(&OpenAlexSavedSearchQuery {
            natural_language_query: "C#".to_string(),
            query_text: "C#".to_string(),
            from_publication_date: None,
            to_publication_date: None,
            work_type: None,
            open_access_only: None,
            search_scope: OpenAlexSearchScope::TitleAndAbstract,
            sort: OpenAlexSort::PublicationDateDesc,
        })
        .await
        .expect("source should resolve");

    assert_ne!(first.source.id, second.source.id);
    assert!(first.source.id.starts_with("openalex:query:c:"));
    assert!(second.source.id.starts_with("openalex:query:c:"));
    assert_eq!(first.source.container_id, first.container.id);
    assert_eq!(second.source.container_id, second.container.id);
}
