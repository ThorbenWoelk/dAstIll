use std::sync::atomic::AtomicU64;
use std::time::Duration;

pub(crate) const CHAT_SOURCE_LIMIT: usize = 6;
pub(crate) const CHAT_SYNTHESIS_SOURCE_LIMIT: usize = 12;
pub(crate) const CHAT_RECOMMENDATION_SOURCE_LIMIT: usize = 14;
pub(crate) const CHAT_PATTERN_SOURCE_LIMIT: usize = 24;
pub(crate) const CHAT_COMPARISON_SOURCE_LIMIT: usize = 20;
pub(crate) const CHAT_HISTORY_LIMIT: usize = 12;
pub(crate) const CHAT_CONTEXT_MAX_CHARS: usize = 1_400;
pub(crate) const CHAT_TITLE_MAX_CHARS: usize = 80;
// Planner calls go through the same cloud-backed prompt path as generation, so
// a 3s budget is too aggressive for non-trivial classification queries.
pub(crate) const CHAT_CLASSIFY_TIMEOUT: Duration = Duration::from_secs(15);
pub(crate) const CHAT_MAX_RETRIEVAL_PASSES: usize = 3;
pub(crate) const CHAT_DIVERSITY_PENALTY: f32 = 0.3;
pub(crate) const CHAT_SOURCE_KIND_DIVERSITY_BONUS: f32 = 1.08;
pub(crate) const CHAT_QUERY_LIMIT_PER_PASS: usize = 3;
pub(crate) const CHAT_QUERY_LIMIT_TOTAL: usize = 5;
pub(crate) const CHAT_RETRIEVAL_CANDIDATE_LIMIT_MIN: usize = 8;
pub(crate) const CHAT_RETRIEVAL_CANDIDATE_LIMIT_MAX: usize = 48;
/// Upper bound for excerpt selection when the user enables deep research (matches retrieval candidate ceiling).
pub(crate) const CHAT_DEEP_RESEARCH_SOURCE_LIMIT: usize = CHAT_RETRIEVAL_CANDIDATE_LIMIT_MAX;
pub(crate) const CHAT_DEEP_RESEARCH_PRIMARY_QUERIES: usize = 6;
pub(crate) const CHAT_DEEP_RESEARCH_EXPANSION_QUERIES: usize = 8;
pub(crate) const CHAT_DEEP_RESEARCH_QUERIES_PER_PASS: usize = 5;
pub(crate) const CHAT_SYNTHESIS_VIDEO_LIMIT: usize = 6;
pub(crate) const CHAT_SYNTHESIS_SOURCES_PER_VIDEO: usize = 3;
pub(crate) const CHAT_SYNTHESIS_CONTEXT_MAX_CHARS: usize = 1_200;

pub(crate) static NEXT_CHAT_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) const CHAT_SYSTEM_PROMPT: &str = "You are the dAstIll assistant. Answer only from the provided ground-truth excerpts and the visible conversation history. If the excerpts are missing, incomplete, or not directly relevant, say that you cannot answer from the current library. Do not use outside knowledge. Do not invent facts, citations, or timestamps. Be concise but useful.\n\nCitation signal (when excerpts are attached): Ground-truth is numbered [Source 1], [Source 2], … in order; each number is one indexed chunk (transcript or summary). For every claim drawn from excerpt N, put the same index in brackets immediately after the words it supports, with no space before the bracket, e.g. …planted a backdoor.[1] or …across two videos.[1][3]. The UI turns each [N] into a link to that chunk; numbers must match the excerpt list.";

pub(crate) const CHAT_SYSTEM_PROMPT_CONVERSATION_TURN: &str = "You are the dAstIll assistant. For this turn, no new transcript excerpts were retrieved. Answer using the visible conversation history and the user's question. If the question clearly requires new evidence from the indexed library, say that briefly. Be concise. Do not invent facts, citations, or timestamps.";

pub(crate) const CHAT_PLANNER_CONVERSATION_MAX_CHARS: usize = 6_000;

pub(crate) const CHAT_QUERY_PLAN_PROMPT: &str = r#"Classify the user's grounded library question for retrieval.

You receive a block labeled RECENT CONVERSATION followed by CURRENT USER MESSAGE.

Return valid JSON only with this shape:
{"needs_retrieval":true|false,"intent":"fact|synthesis|pattern|comparison","rationale":"short explanation","sub_queries":["..."],"expansion_queries":["..."]}

needs_retrieval:
- false only when CURRENT USER MESSAGE can be answered from prior turns without new library search (clarifications, rephrasing, short follow-ups about what was already said).
- true when the user asks for new facts from videos, new topics, comparisons needing evidence, or there is no prior assistant reply to rely on.

If needs_retrieval is false, sub_queries and expansion_queries may be empty arrays.

intent: fact: 1 direct query, no expansion. synthesis: 1-2 queries, optional expansion. pattern/comparison: 2-3 initial queries plus 1-2 expansion queries for broader coverage.

Use the user's wording where possible. Keep each query short. No markdown or code fences."#;

pub(crate) const CHAT_VIDEO_OBSERVATION_PROMPT: &str = "You are distilling grounded evidence for a later answer. Use only the supplied excerpts. Return exactly two concise bullet points describing observations relevant to the user's question. If the excerpts are weak, say that the evidence from this video is limited.";
