use std::collections::{HashMap, HashSet};

use crate::models::ChatSource;
use crate::services::search::{
    SEARCH_RRF_K, SearchCandidate, SearchSourceKind, truncate_chunk_for_display,
};
use crate::services::text::limit_text;

use super::chat::{
    AccumulatedSearchCandidate, CHAT_CONTEXT_MAX_CHARS, CHAT_DIVERSITY_PENALTY,
    CHAT_RETRIEVAL_CANDIDATE_LIMIT_MAX, CHAT_RETRIEVAL_CANDIDATE_LIMIT_MIN,
    CHAT_SOURCE_KIND_DIVERSITY_BONUS, CHAT_SYNTHESIS_SOURCES_PER_VIDEO, ChatQueryIntent,
    ChatRetrievalPlan, CoverageAssessment, RetrievedChatSource, VideoObservationInput,
};
use super::chat_heuristics::preference_signal_score;

pub(super) fn rank_chat_sources(
    candidates: impl IntoIterator<Item = impl std::borrow::Borrow<AccumulatedSearchCandidate>>,
    plan: &ChatRetrievalPlan,
) -> Vec<RetrievedChatSource> {
    let mut remaining = candidates
        .into_iter()
        .map(|candidate| candidate.borrow().clone())
        .filter(|candidate| candidate.combined_score() > 0.0)
        .collect::<Vec<_>>();
    let mut selected = Vec::new();
    let mut video_counts = HashMap::<String, usize>::new();
    let mut kind_counts = HashMap::<SearchSourceKind, usize>::new();

    while selected.len() < plan.budget && !remaining.is_empty() {
        let best_index = remaining
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| {
                selection_score(left, &video_counts, &kind_counts, plan)
                    .total_cmp(&selection_score(right, &video_counts, &kind_counts, plan))
            })
            .map(|(index, _)| index)
            .expect("remaining candidates should not be empty");
        let candidate = remaining.swap_remove(best_index);
        let score = selection_score(&candidate, &video_counts, &kind_counts, plan);
        *video_counts
            .entry(candidate.candidate.video_id.clone())
            .or_insert(0) += 1;
        *kind_counts
            .entry(candidate.candidate.source_kind)
            .or_insert(0) += 1;
        selected.push(RetrievedChatSource {
            source: ChatSource {
                video_id: candidate.candidate.video_id.clone(),
                channel_id: candidate.candidate.channel_id.clone(),
                channel_name: candidate.candidate.channel_name.clone(),
                video_title: candidate.candidate.video_title.clone(),
                source_kind: candidate.candidate.source_kind,
                section_title: candidate.candidate.section_title.clone(),
                snippet: truncate_chunk_for_display(&candidate.candidate.chunk_text),
                score,
                chunk_id: candidate.candidate.chunk_id.clone(),
                retrieval_pass: Some(candidate.retrieval_pass),
            },
            context_text: limit_text(
                candidate.candidate.chunk_text.trim(),
                CHAT_CONTEXT_MAX_CHARS,
            ),
        });
    }

    selected
}

pub(super) fn retrieval_candidate_limit(budget: usize, query_count: usize, pass: usize) -> usize {
    let query_count = query_count.max(1);
    let base = ((budget * 2) / query_count).max(CHAT_RETRIEVAL_CANDIDATE_LIMIT_MIN);
    let boosted = if pass > 1 {
        base + 4 + (pass.saturating_sub(1) * 2)
    } else {
        base
    };
    boosted.clamp(
        CHAT_RETRIEVAL_CANDIDATE_LIMIT_MIN,
        CHAT_RETRIEVAL_CANDIDATE_LIMIT_MAX,
    )
}

pub(super) fn accumulate_ranked_candidates(
    pool: &mut HashMap<String, AccumulatedSearchCandidate>,
    candidates: &[SearchCandidate],
    semantic: bool,
    pass: usize,
) {
    for (index, candidate) in candidates.iter().enumerate() {
        let rank = index + 1;
        let score = 1.0 / (SEARCH_RRF_K + rank as f32);
        let entry =
            pool.entry(candidate.chunk_id.clone())
                .or_insert_with(|| AccumulatedSearchCandidate {
                    candidate: candidate.clone(),
                    keyword_score: 0.0,
                    semantic_score: 0.0,
                    retrieval_pass: pass,
                });
        if semantic {
            entry.semantic_score += score;
        } else {
            entry.keyword_score += score;
        }
        entry.retrieval_pass = entry.retrieval_pass.min(pass);
        entry.candidate = candidate.clone();
    }
}

pub(super) fn assess_coverage(
    plan: &ChatRetrievalPlan,
    sources: &[RetrievedChatSource],
) -> CoverageAssessment {
    if sources.is_empty() {
        return CoverageAssessment {
            needs_more: false,
            reason: Some("No grounded excerpts were found.".to_string()),
            channel_focus_ids: Vec::new(),
        };
    }

    let unique_video_count = count_unique_videos(sources);
    let mut video_counts = HashMap::<String, usize>::new();
    let mut channel_counts = HashMap::<String, usize>::new();
    for source in sources {
        *video_counts
            .entry(source.source.video_id.clone())
            .or_insert(0) += 1;
        *channel_counts
            .entry(source.source.channel_id.clone())
            .or_insert(0) += 1;
    }
    let dominant_video_count = video_counts.values().copied().max().unwrap_or(0);
    let unique_channel_count = channel_counts.len();

    if plan.attributed_preference {
        let direct_evidence_count = count_direct_preference_evidence(sources, &plan.focus_terms);
        let needs_more = plan.supports_second_pass()
            && (direct_evidence_count == 0 || (direct_evidence_count < 2 && sources.len() < 8));
        return CoverageAssessment {
            needs_more,
            reason: Some(if direct_evidence_count == 0 {
                if needs_more {
                    "The current excerpts mention the topic, but they do not yet contain enough direct recommendation-style language, so the search should broaden.".to_string()
                } else {
                    "The current excerpts mention the topic, but they still do not contain a direct recommendation or preference statement.".to_string()
                }
            } else if needs_more {
                format!(
                    "Found {direct_evidence_count} excerpts with direct preference language, but the evidence is still thin enough to justify a broader pass."
                )
            } else {
                format!(
                    "Found {direct_evidence_count} excerpts with direct preference language across {unique_video_count} videos."
                )
            }),
            channel_focus_ids: Vec::new(),
        };
    }

    match plan.intent {
        ChatQueryIntent::Fact => CoverageAssessment {
            needs_more: false,
            reason: Some("Fact lookup stayed focused on the strongest direct matches.".to_string()),
            channel_focus_ids: Vec::new(),
        },
        ChatQueryIntent::Synthesis => {
            let needs_more = sources.len() < 6 && plan.supports_second_pass();
            CoverageAssessment {
                needs_more,
                reason: Some(if needs_more {
                    format!(
                        "Pass 1 found only {} strong excerpts, so broader synthesis coverage is useful.",
                        sources.len()
                    )
                } else {
                    format!(
                        "Pass 1 gathered {} excerpts across {} videos.",
                        sources.len(),
                        unique_video_count
                    )
                }),
                channel_focus_ids: Vec::new(),
            }
        }
        ChatQueryIntent::Pattern => {
            let needs_more = plan.supports_second_pass()
                && (unique_video_count < 4 || dominant_video_count > plan.max_per_video + 1);
            CoverageAssessment {
                needs_more,
                reason: Some(if needs_more {
                    format!(
                        "Pass 1 covered {unique_video_count} videos with heavy concentration in one video, so a broader pass should reduce bias."
                    )
                } else {
                    format!(
                        "Coverage reached {unique_video_count} videos with a balanced spread for pattern analysis."
                    )
                }),
                channel_focus_ids: Vec::new(),
            }
        }
        ChatQueryIntent::Comparison => {
            let dominant_channel_count = channel_counts.values().copied().max().unwrap_or(0);
            let channel_focus_ids = if unique_channel_count > 1 {
                channel_counts
                    .iter()
                    .filter(|(_, count)| **count < dominant_channel_count)
                    .map(|(channel_id, _)| channel_id.clone())
                    .take(2)
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let needs_more = plan.supports_second_pass()
                && (unique_channel_count < 2
                    || dominant_channel_count
                        > (sources.len().saturating_sub(dominant_channel_count) + 2));
            CoverageAssessment {
                needs_more,
                reason: Some(if needs_more {
                    if unique_channel_count < 2 {
                        "Pass 1 did not surface enough distinct channels for a fair comparison."
                            .to_string()
                    } else {
                        "Pass 1 leaned too heavily toward one channel, so the next pass rebalances evidence.".to_string()
                    }
                } else {
                    format!(
                        "Comparison coverage spans {unique_channel_count} channels with enough balance to synthesize."
                    )
                }),
                channel_focus_ids,
            }
        }
        ChatQueryIntent::RecentActivity => {
            let needs_more = plan.supports_second_pass()
                && (unique_video_count < 3 || dominant_video_count > plan.max_per_video + 1);
            CoverageAssessment {
                needs_more,
                reason: Some(if needs_more {
                    format!(
                        "Pass 1 only covered {unique_video_count} recent videos, so a broader pass may improve recent-activity coverage."
                    )
                } else {
                    format!("Recent-activity coverage spans {unique_video_count} recent videos.")
                }),
                channel_focus_ids: Vec::new(),
            }
        }
    }
}

pub(super) fn build_video_observation_inputs(
    sources: &[RetrievedChatSource],
    max_videos: usize,
) -> Vec<VideoObservationInput> {
    let mut groups = Vec::<VideoObservationInput>::new();
    let mut group_indexes = HashMap::<String, usize>::new();

    for source in sources {
        if let Some(index) = group_indexes.get(&source.source.video_id).copied() {
            if groups[index].excerpts.len() < CHAT_SYNTHESIS_SOURCES_PER_VIDEO {
                groups[index].excerpts.push(source.clone());
            }
            continue;
        }

        if groups.len() >= max_videos {
            continue;
        }

        group_indexes.insert(source.source.video_id.clone(), groups.len());
        groups.push(VideoObservationInput {
            video_id: source.source.video_id.clone(),
            video_title: source.source.video_title.clone(),
            channel_name: source.source.channel_name.clone(),
            excerpts: vec![source.clone()],
        });
    }

    groups
}

fn selection_score(
    candidate: &AccumulatedSearchCandidate,
    video_counts: &HashMap<String, usize>,
    kind_counts: &HashMap<SearchSourceKind, usize>,
    plan: &ChatRetrievalPlan,
) -> f32 {
    let mut score = candidate.combined_score();
    let video_count = video_counts
        .get(&candidate.candidate.video_id)
        .copied()
        .unwrap_or(0);
    if video_count >= plan.max_per_video {
        score *= CHAT_DIVERSITY_PENALTY.powi((video_count + 1 - plan.max_per_video) as i32);
    }

    let transcript_count = kind_counts
        .get(&SearchSourceKind::Transcript)
        .copied()
        .unwrap_or(0);
    let summary_count = kind_counts
        .get(&SearchSourceKind::Summary)
        .copied()
        .unwrap_or(0);
    let wants_source_kind_diversity = (transcript_count > 0
        && summary_count == 0
        && candidate.candidate.source_kind == SearchSourceKind::Summary)
        || (summary_count > 0
            && transcript_count == 0
            && candidate.candidate.source_kind == SearchSourceKind::Transcript);
    if wants_source_kind_diversity {
        score *= CHAT_SOURCE_KIND_DIVERSITY_BONUS;
    }

    if plan
        .video_focus_ids
        .iter()
        .any(|video_id| video_id == &candidate.candidate.video_id)
    {
        score *= 1.8;
    } else if plan
        .channel_focus_ids
        .iter()
        .any(|channel_id| channel_id == &candidate.candidate.channel_id)
    {
        score *= 1.2;
    }

    if plan.attributed_preference {
        let preference_score =
            preference_signal_score(&candidate.candidate.chunk_text, &plan.focus_terms);
        if preference_score > 0.0 {
            score *= 1.0 + preference_score;
        }
    }

    score
}

pub(super) fn count_unique_videos(sources: &[RetrievedChatSource]) -> usize {
    sources
        .iter()
        .map(|source| source.source.video_id.as_str())
        .collect::<HashSet<_>>()
        .len()
}

fn count_direct_preference_evidence(
    sources: &[RetrievedChatSource],
    focus_terms: &[String],
) -> usize {
    sources
        .iter()
        .filter(|source| preference_signal_score(&source.context_text, focus_terms) >= 0.14)
        .count()
}
