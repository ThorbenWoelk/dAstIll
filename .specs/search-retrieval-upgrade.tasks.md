# Tasks: Search Retrieval Upgrade

## Current State
Spec created (`search-retrieval-upgrade.md`). Implementation not started.

## Steps

### Phase A - Foundation (High impact)
- [ ] Add Tantivy (or chosen in-process BM25) index module: schema, index build from `search-chunks/` (or streaming load), query API returning ranked chunk ids / scores.
- [ ] Wire keyword search path to use the BM25 index; remove or gate the per-request full scan in `search_fts_candidates`; define startup rebuild and optional incremental invalidation when chunks change.
- [ ] Add `channel_id` to vector index metadata in the search index worker; update `search_vector_candidates` to pass server-side filter; adjust `top_k` logic after filter works; plan backfill for existing vectors.
- [ ] Parse transcript timing from stored transcript format where possible; extend `ChunkDraft` / persisted chunk JSON / vector metadata with optional `start_sec`/`end_sec`; thread through `SearchMatchPayload` (and TS bindings) for UI deep links.
- [ ] Add unit tests for chunk timing parsing and BM25 query ranking smoke tests.

### Phase B - Quality and ops (Medium)
- [ ] Integrate reranker: config (model id, top-K), batch scoring of fused candidates, fallback on error.
- [ ] Implement HyDE behind config: token threshold, prompt, embed hypothetical doc, fallback to raw query embedding.
- [ ] Replace hardcoded hybrid overfetch with policy derived from corpus / status snapshot; document defaults.
- [ ] Add structured logging for top RRF (or fusion) scores per request.

### Phase C - Stretch (Lower priority)
- [ ] Sentence-boundary or semantic-aware transcript chunking; re-validate chunk sizes and overlap.
- [ ] Document and optionally implement configurable embedding dimensions + provider switch with full reindex playbook.

## Decisions Made During Implementation
- (none yet)
