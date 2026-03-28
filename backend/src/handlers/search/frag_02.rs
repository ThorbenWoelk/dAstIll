fn rerank_fts_candidates(candidates: &[SearchCandidate], query: &str) -> Vec<SearchCandidate> {
    let meaningful_terms = meaningful_search_terms(query);
    if candidates.len() <= 1 || meaningful_terms.is_empty() {
        return candidates.to_vec();
    }

    let raw_phrase_tokens = tokenize_search_terms(query);
    let meaningful_phrase_tokens = if meaningful_terms.len() >= 2 {
        Some(meaningful_terms.clone())
    } else {
        None
    };
    let mut ranked = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let exact_phrase_match =
                contains_token_phrase(&candidate.video_title, &raw_phrase_tokens)
                    || contains_token_phrase(&candidate.chunk_text, &raw_phrase_tokens)
                    || candidate
                        .section_title
                        .as_deref()
                        .is_some_and(|title| contains_token_phrase(title, &raw_phrase_tokens))
                    || meaningful_phrase_tokens
                        .as_ref()
                        .is_some_and(|phrase_tokens| {
                            contains_token_phrase(&candidate.video_title, phrase_tokens)
                                || contains_token_phrase(&candidate.chunk_text, phrase_tokens)
                                || candidate.section_title.as_deref().is_some_and(|title| {
                                    contains_token_phrase(title, phrase_tokens)
                                })
                        });
            let title_term_matches =
                count_title_term_matches(&candidate.video_title, &meaningful_terms);
            let title_contains_all_terms = title_term_matches == meaningful_terms.len();
            (
                exact_phrase_match,
                candidate.source_kind == SearchSourceKind::Summary,
                title_contains_all_terms,
                title_term_matches,
                index,
                candidate.clone(),
            )
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| right.2.cmp(&left.2))
            .then_with(|| right.3.cmp(&left.3))
            .then_with(|| left.4.cmp(&right.4))
    });

    ranked
        .into_iter()
        .map(|(_, _, _, _, _, candidate)| candidate)
        .collect()
}

fn group_ranked_candidates(
    candidates: &[SearchCandidate],
    limit: usize,
) -> Vec<SearchVideoResultPayload> {
    let mut grouped = HashMap::<String, SearchVideoResultPayload>::new();
    let mut best_ranks = HashMap::<String, usize>::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let rank = index + 1;
        let group = grouped
            .entry(candidate.video_id.clone())
            .or_insert_with(|| SearchVideoResultPayload {
                video_id: candidate.video_id.clone(),
                channel_id: candidate.channel_id.clone(),
                channel_name: candidate.channel_name.clone(),
                video_title: candidate.video_title.clone(),
                published_at: candidate.published_at.clone(),
                matches: Vec::new(),
            });

        let existing = group
            .matches
            .iter()
            .position(|existing| existing.source == candidate.source_kind);
        let score = 1.0 / (SEARCH_RRF_K + rank as f32);
        let payload = SearchMatchPayload {
            source: candidate.source_kind,
            section_title: candidate.section_title.clone(),
            snippet: truncate_chunk_for_display(&candidate.chunk_text),
            score,
            start_sec: candidate.start_sec,
        };

        match existing {
            Some(index) if payload.score > group.matches[index].score => {
                group.matches[index] = payload;
            }
            None => group.matches.push(payload),
            _ => {}
        }

        best_ranks
            .entry(candidate.video_id.clone())
            .and_modify(|best| *best = (*best).min(rank))
            .or_insert(rank);
    }

    let mut results = grouped.into_values().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        let left_rank = best_ranks
            .get(&left.video_id)
            .copied()
            .unwrap_or(usize::MAX);
        let right_rank = best_ranks
            .get(&right.video_id)
            .copied()
            .unwrap_or(usize::MAX);
        left_rank
            .cmp(&right_rank)
            .then_with(|| right.published_at.cmp(&left.published_at))
    });
    for result in &mut results {
        result.matches.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
        });
    }
    results.truncate(limit);
    results
}

fn group_fts_candidates(
    candidates: &[SearchCandidate],
    limit: usize,
) -> Vec<SearchVideoResultPayload> {
    group_ranked_candidates(candidates, limit)
}

/// Merge vector and FTS candidate lists via RRF, returning a flat deduplicated list
/// ordered by descending fused score. Used as input to the cross-encoder reranker.
fn collect_rrf_candidates(
    vector_candidates: &[SearchCandidate],
    fts_candidates: &[SearchCandidate],
) -> Vec<SearchCandidate> {
    let vector_ranks: Vec<(&str, usize)> = vector_candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (c.chunk_id.as_str(), i + 1))
        .collect();
    let fts_ranks: Vec<(&str, usize)> = fts_candidates
        .iter()
        .enumerate()
        .map(|(i, c)| (c.chunk_id.as_str(), i + 1))
        .collect();
    let fused = fuse_ranked_matches(&vector_ranks, &fts_ranks, SEARCH_RRF_K);

    let mut by_id: std::collections::HashMap<&str, &SearchCandidate> =
        std::collections::HashMap::new();
    for c in vector_candidates.iter().chain(fts_candidates.iter()) {
        by_id.insert(c.chunk_id.as_str(), c);
    }

    fused
        .into_iter()
        .filter_map(|(chunk_id, _score)| by_id.get(chunk_id.as_str()).copied().cloned())
        .collect()
}

fn rank_and_group_candidates(
    vector_candidates: &[SearchCandidate],
    fts_candidates: &[SearchCandidate],
    limit: usize,
) -> Vec<SearchVideoResultPayload> {
    let vector_ranks = vector_candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate.chunk_id.as_str(), index + 1))
        .collect::<Vec<_>>();
    let fts_ranks = fts_candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate.chunk_id.as_str(), index + 1))
        .collect::<Vec<_>>();
    let fused = fuse_ranked_matches(&vector_ranks, &fts_ranks, SEARCH_RRF_K);

    let mut candidates = HashMap::<String, SearchCandidate>::new();
    for candidate in vector_candidates.iter().chain(fts_candidates.iter()) {
        candidates.insert(candidate.chunk_id.clone(), candidate.clone());
    }

    let mut grouped = HashMap::<String, SearchVideoResultPayload>::new();
    let mut best_scores = HashMap::<String, f32>::new();

    for (chunk_id, score) in fused {
        let Some(candidate) = candidates.get(&chunk_id) else {
            continue;
        };

        let group = grouped
            .entry(candidate.video_id.clone())
            .or_insert_with(|| SearchVideoResultPayload {
                video_id: candidate.video_id.clone(),
                channel_id: candidate.channel_id.clone(),
                channel_name: candidate.channel_name.clone(),
                video_title: candidate.video_title.clone(),
                published_at: candidate.published_at.clone(),
                matches: Vec::new(),
            });

        let existing = group
            .matches
            .iter()
            .position(|existing| existing.source == candidate.source_kind);
        let payload = SearchMatchPayload {
            source: candidate.source_kind,
            section_title: candidate.section_title.clone(),
            snippet: truncate_chunk_for_display(&candidate.chunk_text),
            score,
            start_sec: candidate.start_sec,
        };

        match existing {
            Some(index) if payload.score > group.matches[index].score => {
                group.matches[index] = payload;
            }
            None => group.matches.push(payload),
            _ => {}
        }

        best_scores
            .entry(candidate.video_id.clone())
            .and_modify(|best| *best = best.max(score))
            .or_insert(score);
    }

    let mut results = grouped.into_values().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        let right_score = best_scores
            .get(&right.video_id)
            .copied()
            .unwrap_or_default();
        let left_score = best_scores.get(&left.video_id).copied().unwrap_or_default();
        right_score
            .partial_cmp(&left_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.published_at.cmp(&left.published_at))
    });
    for result in &mut results {
        result.matches.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
        });
    }
    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use super::{
        SearchExecutionMode, SearchRetrievalMode, SearchSourceFilter, group_fts_candidates,
        group_ranked_candidates, rank_and_group_candidates, rerank_fts_candidates,
        resolve_requested_retrieval_mode, resolve_search_retrieval_mode,
        resolve_semantic_exact_source_kind, resolve_semantic_retrieval_mode,
    };
    use crate::search_query::build_fts_query;
    use crate::services::search::{SearchCandidate, SearchSourceKind};

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
}
