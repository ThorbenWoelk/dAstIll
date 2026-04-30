use super::*;

fn count_candidate_term_matches(candidate: &SearchCandidate, terms: &[String]) -> usize {
    let mut candidate_terms = tokenize_search_terms(&candidate.video_title)
        .into_iter()
        .collect::<std::collections::HashSet<_>>();

    if let Some(section_title) = candidate.section_title.as_deref() {
        candidate_terms.extend(tokenize_search_terms(section_title));
    }
    candidate_terms.extend(tokenize_search_terms(&candidate.chunk_text));

    terms
        .iter()
        .filter(|term| candidate_terms.contains(*term))
        .count()
}

pub(super) fn rerank_fts_candidates(
    candidates: &[SearchCandidate],
    query: &str,
) -> Vec<SearchCandidate> {
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
            let candidate_term_matches = count_candidate_term_matches(candidate, &meaningful_terms);
            let candidate_contains_all_terms = candidate_term_matches == meaningful_terms.len();
            (
                exact_phrase_match,
                candidate.source_kind == SearchSourceKind::Summary,
                candidate_contains_all_terms,
                candidate_term_matches,
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
            .then_with(|| right.4.cmp(&left.4))
            .then_with(|| right.5.cmp(&left.5))
            .then_with(|| left.6.cmp(&right.6))
    });

    ranked
        .into_iter()
        .map(|(_, _, _, _, _, _, _, candidate)| candidate)
        .collect()
}

pub(super) fn group_ranked_candidates(
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
                source_id: candidate.channel_id.clone(),
                video_id: candidate.video_id.clone(),
                item_id: candidate.video_id.clone(),
                provider: crate::models::infer_provider_kind_for_source_id(&candidate.channel_id),
                source_kind: crate::models::infer_source_kind_for_source_id(&candidate.channel_id),
                item_kind: crate::models::infer_item_kind_for_source_kind(
                    crate::models::infer_source_kind_for_source_id(&candidate.channel_id),
                ),
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

pub(super) fn group_fts_candidates(
    candidates: &[SearchCandidate],
    limit: usize,
) -> Vec<SearchVideoResultPayload> {
    group_ranked_candidates(candidates, limit)
}

/// Merge vector and FTS candidate lists via RRF, returning a flat deduplicated list
/// ordered by descending fused score. Used as input to the cross-encoder reranker.
pub(super) fn collect_rrf_candidates(
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

pub(super) fn rank_and_group_candidates(
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
                source_id: candidate.channel_id.clone(),
                video_id: candidate.video_id.clone(),
                item_id: candidate.video_id.clone(),
                provider: crate::models::infer_provider_kind_for_source_id(&candidate.channel_id),
                source_kind: crate::models::infer_source_kind_for_source_id(&candidate.channel_id),
                item_kind: crate::models::infer_item_kind_for_source_kind(
                    crate::models::infer_source_kind_for_source_id(&candidate.channel_id),
                ),
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
#[path = "ranking_tests.rs"]
mod ranking_tests;
