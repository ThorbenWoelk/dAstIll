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
pub(crate) const CHAT_TOOL_LOOP_MAX_STEPS: usize = 4;
pub(crate) const CHAT_TOOL_LOOP_MAX_STEPS_DEEP_RESEARCH: usize = 6;

pub(crate) static NEXT_CHAT_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) const CHAT_SYSTEM_PROMPT: &str = "You are the dAstIll assistant. Answer only from the provided ground-truth excerpts, tool outputs, and the visible conversation history. If the evidence is missing, incomplete, or not directly relevant, say so clearly. Do not use outside knowledge. Do not invent facts, citations, or timestamps. Be concise but useful.\n\nCitation signal (when excerpts are attached): Ground-truth excerpts are numbered [Source 1], [Source 2], … in order; each number is one indexed chunk (transcript or summary). For every claim drawn from excerpt N, put the same index in brackets immediately after the words it supports, with no space before the bracket, e.g. …planted a backdoor.[1] or …across two videos.[1][3]. The UI turns each [N] into a link to that chunk; numbers must match the excerpt list. Tool outputs are already trusted app data and do not need citation markers unless also supported by excerpts.";

pub(crate) const CHAT_SYSTEM_PROMPT_CONVERSATION_TURN: &str = "You are the dAstIll assistant. For this turn, no new transcript excerpts were retrieved. Answer using the visible conversation history and the user's question. If the question clearly requires new evidence from the indexed library, say that briefly. Be concise. Do not invent facts, citations, or timestamps.";

pub(crate) const CHAT_PLANNER_CONVERSATION_MAX_CHARS: usize = 6_000;

pub(crate) const CHAT_QUERY_PLAN_PROMPT: &str = r#"Classify the user's message for whether the indexed video library must be searched on this turn.

You receive a block labeled RECENT CONVERSATION (possibly empty) followed by CURRENT USER MESSAGE.

Return valid JSON only with this shape:
{"needs_retrieval":true|false,"intent":"fact|synthesis|pattern|comparison","rationale":"short explanation","sub_queries":["..."],"expansion_queries":["..."]}

needs_retrieval:
- false when no new library search is needed: (a) clarifications or follow-ups that only rely on what was already said in RECENT CONVERSATION, or (b) pure greetings, thanks, goodbyes, or other small talk with no question about video, channel, or transcript content. Use false for (b) even when RECENT CONVERSATION is empty (first message).
- true when the user wants facts, summaries, themes, or comparisons from the indexed library, names a topic to look up, or otherwise clearly needs grounded excerpts.

If needs_retrieval is false, sub_queries and expansion_queries may be empty arrays.

intent: fact: 1 direct query, no expansion. synthesis: 1-2 queries, optional expansion. pattern/comparison: 2-3 initial queries plus 1-2 expansion queries for broader coverage.

Use the user's wording where possible. Keep each query short. No markdown or code fences."#;

pub(crate) const CHAT_TOOL_LOOP_PROMPT: &str = r#"You are controlling the next step in dAstIll chat. Decide whether to answer from the current evidence or call one safe tool.

You receive:
- RECENT CONVERSATION
- CURRENT USER MESSAGE
- TOOL RESULTS FROM THIS TURN

Available tools:

1. search_library
- Use for questions about transcript or summary content, themes, comparisons, recommendations, or grounded evidence from the indexed library.
- The backend handles keyword search, semantic search, candidate fusion, and ranking internally.
- Input JSON:
  {"query":"short search query","source":"all|summary|transcript","limit":1-24}

2. db_inspect
- Use for read-only questions about stored app data itself, such as counts or small sample lists.
- Input JSON:
  {"operation":"count|list","resource":"summaries|transcripts|videos|channels","limit":1-10}

Return valid JSON only with this shape:
{"action":"respond|tool_call","rationale":"short explanation","tool_name":"search_library|db_inspect"|null,"search_library_input":{"query":"...","source":"all|summary|transcript","limit":1-24}|null,"db_inspect_input":{"operation":"count|list","resource":"summaries|transcripts|videos|channels","limit":1-10}|null}

Rules:
- Prefer responding when the current conversation and tool results already provide enough information.
- Call at most one tool per response.
- Use search_library instead of trying to reason about retrieval strategy yourself.
- Use db_inspect only for read-only stored-data questions.
- Do not invent tools or arguments outside the allowed schemas.
- Keep search_library queries short and broad.
- If the user is greeting, thanking, or making small talk, respond.
- No markdown or code fences."#;

pub(crate) const CHAT_VIDEO_OBSERVATION_PROMPT: &str = "You are distilling grounded evidence for a later answer. Use only the supplied excerpts. Return exactly two concise bullet points describing observations relevant to the user's question. If the excerpts are weak, say that the evidence from this video is limited.";
