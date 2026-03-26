# Tasks: Search Architecture Improvements

## Current State
Features 1-3 and 6 complete. Neural reranker (4) and HyDE (5) pending.

## Steps

### Feature 3: channel_id server-side S3 Vectors filter
- [x] Add `channel_id` to `SearchMaterial` in `db/mod.rs`
- [x] Populate `channel_id` in `load_search_material` from `video.channel_id`
- [x] Add `channel_id: &str` param to `replace_search_chunks`; store in metadata
- [x] Apply server-side channel filter in `search_vector_candidates`; remove 3x over-fetch
- [x] Update `workers/search_index.rs` to pass `material.channel_id`

### Feature 2: Timestamp preservation in transcript chunks
- [x] Add `TimedSegment` struct and `timed_text` field to `Transcript` in `models.rs`
- [x] Add `start_sec: Option<f32>` to `SearchMatchPayload` in `models.rs`
- [x] Update `parse_json3_transcript` to return `Vec<TimedSegment>` (with `tStartMs`)
- [x] Update `extract_with_ytdlp` to populate `Transcript.timed_text`
- [x] Add `start_sec: Option<f32>` to `ChunkDraft` and `SearchIndexChunk`
- [x] Update `chunk_transcript_content` to accept `Option<&[TimedSegment]>` and set `start_sec`
- [x] Store/retrieve `start_sec` in S3 chunk JSON and S3 Vectors metadata
- [x] Thread `start_sec` through `SearchCandidate` -> grouping -> `SearchMatchPayload`
- [x] Pass `timed_text` from transcript in `workers/search_index.rs`

### Feature 1: In-memory BM25 index (Tantivy)
- [x] Add `tantivy` to `Cargo.toml`
- [x] Create `services/fts.rs` with `FtsIndex` (build, upsert_source, delete_source, search)
- [x] Export `FtsIndex` from `services/mod.rs`
- [x] Add `fts: Arc<FtsIndex>` to `AppState`
- [x] Initialize FTS index at startup in `main.rs`; populate from S3 chunks via `populate_fts_index_from_store`
- [x] Update search index worker: `upsert_source` after write, `delete_source` after clear
- [x] Replace `db::search_fts_candidates` in `handlers/search.rs` with `state.fts.search`
- [x] Replace `db::search_fts_candidates` in `services/chat/mod.rs` with `state.fts.search`
- [x] Remove `search_fts_candidates` from `db/search.rs`
- [x] Add `published_at` to `SearchMaterial` for FTS index hydration
- [x] 271 tests pass

### Feature 6: Timed semantic chunking
- [x] Implement `chunk_transcript_timed` in `services/search.rs`
- [x] Call it from `chunk_transcript_content` when timed segments are available

### Feature 4: Neural reranker (Ollama)
- [x] Add `SEARCH_RERANK_MODEL` to `OllamaRuntimeConfig`
- [x] Add `rerank_model` to `SearchService`; add `rerank_candidates` method (POST `/api/rerank`)
- [x] Wire rerank model into `SearchService` construction in `main.rs`
- [x] Apply reranker after RRF fusion in hybrid mode; `collect_rrf_candidates` helper for merge
- [x] Log `rerank_configured`, `rerank_elapsed_ms` in search log line

### Feature 5: HyDE for short queries
- [x] Add `SEARCH_HYDE_MODEL` to `OllamaRuntimeConfig`
- [x] Add `hyde_model` to `SearchService`; add `generate_hyde_passage` method (POST `/api/generate`)
- [x] Apply HyDE in handler for queries with ≤4 meaningful tokens when configured
- [x] Log `hyde_triggered`, `hyde_elapsed_ms` in search log line

### Final verification
- [x] Run full `cargo test` suite (271 passed)
- [x] Run pre-commit checks on all modified files

## Decisions Made During Implementation
- Feature 3 first: smallest change, isolated, unlocks perf improvement for channel-scoped search immediately
- Feature 2 second: no hard deps, required before Feature 6
- Feature 1 (Tantivy): most impactful FTS improvement; after 2 so new `start_sec` field is included in the FTS index schema
- Features 4+5 last: optional enhancements, no structural deps
- Feature 6 last: depends on Feature 2 (timed segments)
- `channel_id` server-side filter: old chunks (without `channel_id` in metadata) continue to work via client-side fallback - no re-indexing required on deploy
- `f_source_key` in Tantivy schema uses `raw` tokenizer (keyword field) so term-based deletion works precisely per video+source_kind without touching the other source
- `populate_fts_index_from_store` takes `AppState` by value (not ref) so it can be moved into `tokio::spawn`
- `published_at` added to `SearchMaterial` so FTS results carry it through to `SearchCandidate` for sort ordering
