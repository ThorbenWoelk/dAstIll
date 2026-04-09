# Fix Keyword Search

## Context
- **Problem**: Keyword search queries like "claude" return zero results even though the term should exist in the transcript and summary data. Investigation needed to identify whether the issue is in tokenization, FTS index content, index sync, or query execution.
- **Goal**: Keyword search must return relevant results for terms that exist in the indexed content. The search path must work correctly before any RAG improvements can be measured.
- **Linear**: N/A

## Implementation Plan
- [ ] **Phase 1: Diagnosis.** Investigate why keyword search fails for valid terms.
  - Run integration test with real data to reproduce the failure.
  - Check FTS index population: Does `fts_search` table contain the expected chunks?
  - Check FTS query generation: Is `build_fts_query("claude")` producing `'"claude"'`?
  - Check FTS execution: Is the SQLite FTS5 MATCH query returning results locally?
  - Check index sync: Is `search_index` worker populating the FTS tables from Turso transcripts?
- [ ] **Phase 2: Root cause isolation.** Based on diagnosis, identify the specific failure point:
  - Empty index: Data not synced from remote DB to local FTS.
  - Tokenization bug: Query terms filtered incorrectly.
  - FTS query syntax: MATCH clause malformed.
  - Access control: Results filtered out by `filter_search_candidates_for_access`.
- [ ] **Phase 3: Fix.** Implement targeted fix for identified root cause.
- [ ] **Phase 4: Regression hardening.** Add test that asserts search returns results for known-good terms in seeded data.

## Requirements
- [ ] **Requirement 1**: Search queries for terms present in indexed content return non-empty results. -> Verification: Integration test searches for known term in seeded transcripts/summaries and receives results.
- [ ] **Requirement 2**: Tokenization does not incorrectly filter valid terms. -> Verification: Unit test asserts `meaningful_search_terms("claude")` returns `["claude"]`.
- [ ] **Requirement 3**: FTS index is populated with content from videos/channels accessible to the user. -> Verification: Integration test checks FTS table has chunks for accessible channels after index build.
- [ ] **Requirement 4**: Search results respect user access scope. -> Verification: Integration test shows search returns only videos from `allowed_channel_ids` or videos in `allowed_other_video_ids`.

## Verification Gates
- [ ] **TDD**: Write red test demonstrating search failure for a known-good term. Keep failing until fix lands.
- [ ] **CSO**: STRIDE audit to ensure search does not leak private channel data. Verify `AccessContext` filtering is applied.
- [ ] **Design Review**: Confirm tokenization rules (stopwords, short terms) match user expectations.
- [ ] **Success**: Evidence (test output, query logs, FTS table inspection) showing search works for known terms.

## Anti-Rationalization (Blocked Excuses)
- "The index might not have the data yet." — That's the bug. Either index sync is broken or the data exists and search is broken. Either way, investigate and fix.
- "Search needs a better algorithm." — Not the issue. Fix the baseline keyword search first. Semantic/RAG improvements are separate.
- "Users should use natural language queries." — Keyword search is the foundation. If it doesn't work, RAG is broken too.

## Technical Notes

### Search Pipeline
1. **User query** → `meaningful_search_terms()` → filtered tokens (max 4)
2. **FTS query** → `build_fts_query()` → `"term1" AND "term2"` format
3. **Execution** → `FtsIndex::search()` → BM25-ranked results
4. **Access filtering** → `filter_search_candidates_for_access()` → scoped results

### Tokenization Rules (backend/src/search_query.rs)
```rust
// Stopwords filtered out:
const SEARCH_STOPWORDS: &[&str] = &[
    "a", "an", "and", "best", "for", "how", "in", "is", "of", "on", "or", 
    "the", "to", "what", "which"
];

// Short technical terms kept even if len < 3:
const SHORT_TECHNICAL_SEARCH_TERMS: &[&str] = &["ai", "db", "go", "js", "ml", "ui", "ux"];

// Meaningful terms must be:
// - len >= 2
// - not in SEARCH_STOPWORDS
// - len >= 3 OR in SHORT_TECHNICAL_SEARCH_TERMS
```

### Potential Failure Points

#### 1. Empty FTS Index
The FTS index lives in a local SQLite file (`search-fts.db`). If the index worker hasn't populated it from the remote Turso DB, search returns nothing.
- **Check**: `backend/src/workers/search_index.rs` - is the worker running?
- **Check**: Does `fts_search` table have rows?
- **Check**: Is `build_index_from_turso()` being called?

#### 2. Tokenization Bug
"claude" has length 6, is not a stopword, and is not in `SHORT_TECHNICAL_SEARCH_TERMS`. It should pass `is_meaningful_search_term()`.
- **Check**: Unit test for `meaningful_search_terms("claude")` → `["claude"]`

#### 3. FTS Query Syntax
The query format is `"term"` for single terms. FTS5 expects this syntax for phrase matching.
- **Check**: Manual query against FTS DB: `SELECT * FROM fts_search WHERE fts_search MATCH '"claude"' LIMIT 10;`

#### 4. Access Filtering Over-Aggressive
`filter_search_candidates_for_access()` removes results the user cannot access. If `AccessContext` is empty or malformed, all results get filtered.
- **Check**: Is `allowed_channel_ids` populated for anonymous users? (Should include seeded "Other" channel)
- **Check**: Is `allowed_other_video_ids` populated correctly?

### Debugging Commands
```bash
# Check FTS index content
sqlite3 /path/to/search-fts.db "SELECT COUNT(*) FROM fts_search;"
sqlite3 /path/to/search-fts.db "SELECT COUNT(*) FROM fts_search WHERE fts_search MATCH 'claude';"

# Check tokenization
cargo test meaningful_search_terms

# Check access context for anonymous user
# Inspect logs for AccessContext construction in handlers/chat.rs
```

## Non-Goals
- Semantic search improvements (separate work).
- RAG ranking algorithm changes (separate work).
- Adding new search features like fuzzy matching, stemming, or typo tolerance.