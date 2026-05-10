use super::{
    group_fts_candidates, group_ranked_candidates, rank_and_group_candidates, rerank_fts_candidates,
};
use crate::search::handler::{
    SearchExecutionMode, SearchRetrievalMode, SearchSourceFilter, resolve_requested_retrieval_mode,
    resolve_semantic_exact_source_kind, resolve_semantic_retrieval_mode,
};
use crate::search::query::build_fts_query;
use crate::search::{SearchCandidate, SearchSourceKind};

fn candidate(chunk_id: &str, video_id: &str, source_kind: SearchSourceKind) -> SearchCandidate {
    SearchCandidate {
        chunk_id: chunk_id.to_string(),
        video_id: video_id.to_string(),
        channel_id: "channel".to_string(),
        channel_name: "Channel".to_string(),
        video_title: "Title".to_string(),
        source_kind,
        section_title: None,
        chunk_text: "A detailed snippet about semantic search.".to_string(),
        published_at: "2026-03-12T00:00:00Z".to_string(),
        start_sec: None,
    }
}

fn resolve_search_retrieval_mode(
    embeddings_available: bool,
    vector_index_ready: bool,
) -> SearchRetrievalMode {
    if !embeddings_available {
        SearchRetrievalMode::FtsOnly
    } else if vector_index_ready {
        SearchRetrievalMode::HybridAnn
    } else {
        SearchRetrievalMode::HybridExact
    }
}

#[test]
fn build_fts_query_quotes_search_terms() {
    assert_eq!(
        build_fts_query("semantic search qwen3-embedding"),
        "\"semantic\" AND \"search\" AND \"qwen3-embedding\""
    );
}

#[test]
fn build_fts_query_drops_broad_question_stopwords_but_keeps_technical_terms() {
    assert_eq!(
        build_fts_query("what is the best db in town"),
        "\"db\" AND \"town\""
    );
    assert_eq!(build_fts_query("how to use ai"), "\"use\" AND \"ai\"");
}

#[test]
fn build_fts_query_deduplicates_and_caps_terms() {
    assert_eq!(
        build_fts_query("rust rust tokio axum libsql semantic search"),
        "\"rust\" AND \"tokio\" AND \"axum\" AND \"libsql\""
    );
}

#[test]
fn rerank_fts_candidates_prioritizes_phrase_then_summary_then_title() {
    let results = rerank_fts_candidates(
        &[
            SearchCandidate {
                video_title: "town database guide".to_string(),
                source_kind: SearchSourceKind::Transcript,
                chunk_text: "db town".to_string(),
                ..candidate("a", "video-1", SearchSourceKind::Transcript)
            },
            SearchCandidate {
                video_title: "DB choices".to_string(),
                source_kind: SearchSourceKind::Summary,
                chunk_text: "database options across the town with db comparisons".to_string(),
                ..candidate("b", "video-2", SearchSourceKind::Summary)
            },
            SearchCandidate {
                video_title: "Other video".to_string(),
                source_kind: SearchSourceKind::Transcript,
                chunk_text: "a db for every town".to_string(),
                ..candidate("c", "video-3", SearchSourceKind::Transcript)
            },
        ],
        "db town",
    );

    assert_eq!(results[0].video_id, "video-1");
    assert_eq!(results[1].video_id, "video-2");
    assert_eq!(results[2].video_id, "video-3");
}

#[test]
fn rerank_fts_candidates_prefers_candidates_covering_more_terms_across_fields() {
    let results = rerank_fts_candidates(
        &[
            SearchCandidate {
                video_title: "What's a Hard Fork?".to_string(),
                source_kind: SearchSourceKind::Summary,
                chunk_text: "A short explainer for the Hard Fork podcast.".to_string(),
                ..candidate("a", "video-1", SearchSourceKind::Summary)
            },
            SearchCandidate {
                video_title: "Anthropic shock wave + One Good Thing".to_string(),
                source_kind: SearchSourceKind::Summary,
                chunk_text: "This Hard Fork episode closes with One Good Thing.".to_string(),
                ..candidate("b", "video-2", SearchSourceKind::Summary)
            },
        ],
        "that hard fork episode with one good thing",
    );

    assert_eq!(results[0].video_id, "video-2");
    assert_eq!(results[1].video_id, "video-1");
}

#[test]
fn grouping_keeps_best_match_per_source_kind() {
    let results = rank_and_group_candidates(
        &[
            candidate("a", "video-1", SearchSourceKind::Summary),
            candidate("b", "video-1", SearchSourceKind::Transcript),
        ],
        &[candidate("b", "video-1", SearchSourceKind::Transcript)],
        10,
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].matches.len(), 2);
}

#[test]
fn source_filter_all_maps_to_no_db_filter() {
    assert_eq!(SearchSourceFilter::All.as_source_kind(), None);
    assert_eq!(
        SearchSourceFilter::Summary.as_source_kind(),
        Some(SearchSourceKind::Summary)
    );
}

#[test]
fn semantic_exact_fallback_prefers_summaries_for_all_sources() {
    assert_eq!(
        resolve_semantic_exact_source_kind(SearchSourceFilter::All),
        Some(SearchSourceKind::Summary)
    );
    assert_eq!(
        resolve_semantic_exact_source_kind(SearchSourceFilter::Transcript),
        Some(SearchSourceKind::Transcript)
    );
}

#[test]
fn fts_grouping_preserves_bm25_rank_order() {
    let results = group_fts_candidates(
        &[
            candidate("a", "video-1", SearchSourceKind::Summary),
            candidate("b", "video-2", SearchSourceKind::Transcript),
            candidate("c", "video-1", SearchSourceKind::Transcript),
        ],
        10,
    );

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].video_id, "video-1");
    assert_eq!(results[0].matches.len(), 2);
    assert_eq!(results[1].video_id, "video-2");
}

#[test]
fn fts_grouping_respects_limit() {
    let results = group_fts_candidates(
        &[
            candidate("a", "video-1", SearchSourceKind::Summary),
            candidate("b", "video-2", SearchSourceKind::Transcript),
        ],
        1,
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].video_id, "video-1");
}

#[test]
fn semantic_grouping_preserves_rank_order() {
    let results = group_ranked_candidates(
        &[
            candidate("a", "video-2", SearchSourceKind::Summary),
            candidate("b", "video-1", SearchSourceKind::Transcript),
        ],
        10,
    );

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].video_id, "video-2");
}

#[test]
fn retrieval_mode_falls_back_to_fts_without_vector_index() {
    assert_eq!(
        resolve_search_retrieval_mode(false, false).as_str(),
        "fts_only"
    );
    assert_eq!(
        resolve_search_retrieval_mode(true, false).as_str(),
        "hybrid_exact"
    );
    assert_eq!(
        resolve_search_retrieval_mode(true, true).as_str(),
        "hybrid_ann"
    );
}

#[test]
fn semantic_retrieval_mode_is_disabled_when_semantic_search_is_not_configured() {
    assert_eq!(resolve_semantic_retrieval_mode(false, false), None);
    assert_eq!(
        resolve_semantic_retrieval_mode(true, false),
        Some(SearchRetrievalMode::HybridExact)
    );
    assert_eq!(
        resolve_semantic_retrieval_mode(true, true),
        Some(SearchRetrievalMode::HybridAnn)
    );
}

#[test]
fn keyword_mode_forces_fts_only_even_when_hybrid_is_ready() {
    assert!(SearchExecutionMode::Keyword.runs_keyword());
    assert!(!SearchExecutionMode::Keyword.runs_semantic());
    assert!(!SearchExecutionMode::Semantic.runs_keyword());
    assert!(SearchExecutionMode::Semantic.runs_semantic());
    assert!(SearchExecutionMode::Hybrid.runs_keyword());
    assert!(SearchExecutionMode::Hybrid.runs_semantic());
    assert_eq!(
        resolve_requested_retrieval_mode(SearchExecutionMode::Keyword, true, true),
        SearchRetrievalMode::FtsOnly,
    );
    assert_eq!(
        resolve_requested_retrieval_mode(SearchExecutionMode::Semantic, true, true),
        SearchRetrievalMode::HybridAnn,
    );
    assert_eq!(
        resolve_requested_retrieval_mode(SearchExecutionMode::Hybrid, true, false),
        SearchRetrievalMode::HybridExact,
    );
}
