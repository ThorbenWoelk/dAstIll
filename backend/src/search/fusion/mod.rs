use std::cmp::Ordering;
use std::collections::HashMap;

/// Reciprocal Rank Fusion constant - balances the contribution of low-ranked results.
pub const SEARCH_RRF_K: f32 = 60.0;

/// Combine two ranked result lists using Reciprocal Rank Fusion.
///
/// Each `(chunk_id, rank)` pair contributes `1 / (rrf_k + rank)` to the fused
/// score. Results are returned sorted by descending score, with ties broken
/// lexicographically by chunk_id for determinism.
pub fn fuse_ranked_matches(
    vector_ranks: &[(&str, usize)],
    fts_ranks: &[(&str, usize)],
    rrf_k: f32,
) -> Vec<(String, f32)> {
    let mut fused = HashMap::<String, f32>::new();

    for (chunk_id, rank) in vector_ranks.iter().chain(fts_ranks.iter()) {
        let score = 1.0 / (rrf_k + (*rank as f32));
        *fused.entry((*chunk_id).to_string()).or_default() += score;
    }

    let mut entries = fused.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    entries
}
