# Search Architecture Improvements

## Background

The current search pipeline:
- **Retrieval**: FTS (full S3 scan + substring match) + S3 Vectors ANN, fused with RRF
- **Chunking**: 300-word sliding window with 40-word overlap (transcript); markdown-section-aware (summary)
- **Embedding**: Ollama, 512 dims, enriched with video/channel/source metadata
- **Reranking**: Heuristic only - exact phrase match > summary source > title term coverage

Six concrete improvements are in scope. Implementation order follows dependency graph and impact/effort ratio.

---

## Feature 1: In-memory BM25 Index (Tantivy)

### Problem

`db::search_fts_candidates` (`db/search.rs:634`) loads **every** chunk JSON object from S3 on every keyword search, then filters in Rust by plain substring match, sorted by raw token count. This is O(n) over the full corpus with no BM25 scoring - no TF-IDF, no field-length normalization.

### Design

Add an in-process Tantivy index (`services/fts.rs`) held in `AppState`. Fields: `chunk_id`, `video_id`, `channel_id`, `source_kind`, `section_title`, `chunk_text`, `video_title`. The index lives in memory and is rebuilt from all S3 chunk objects at startup.

The search index worker (`workers/search_index.rs`) updates the FTS index after each successful `replace_search_chunks` call. When chunks are deleted (source cleared or rebuilt), the corresponding documents are deleted from the index.

The handler calls `state.fts.search(...)` instead of `db::search_fts_candidates`. The `db::search_fts_candidates` function is removed.

**Score translation**: Tantivy returns BM25 scores. These are normalized to a `[0, 1]` rank-based score before RRF fusion (same as the vector path - rank position matters, not raw score).

**Startup**: Load all `search-chunks/*.json` objects concurrently (same concurrency cap as current: 24), index them, then mark index as ready. Searches issued before the index is ready fall back to an empty FTS result (hybrid mode still returns vector-only results).

### Files

- `backend/Cargo.toml` - add `tantivy`
- `backend/src/services/fts.rs` (new) - `FtsIndex` struct: `build_from_chunks`, `upsert_source`, `delete_source`, `search`
- `backend/src/services/mod.rs` - pub use FtsIndex
- `backend/src/state.rs` - `fts: Arc<FtsIndex>`
- `backend/src/main.rs` - initialize and populate FtsIndex at startup
- `backend/src/workers/search_index.rs` - call `state.fts.upsert_source(...)` after successful DB write; `delete_source` after clear
- `backend/src/handlers/search.rs` - replace `db::search_fts_candidates` call with `state.fts.search(...)`
- `backend/src/db/search.rs` - remove `search_fts_candidates`

### Acceptance criteria

- Keyword search returns BM25-ranked results, not token-count sorted
- Channel-scoped keyword search works (FTS accepts optional `channel_id` filter)
- Source-kind filter works
- After a new video is indexed, it appears in FTS results without a restart
- All existing keyword/hybrid search tests pass with the new index

---

## Feature 2: Timestamp Preservation in Transcript Chunks

### Problem

The yt-dlp json3 fallback path (`services/transcript.rs:parse_json3_transcript`) discards `tStartMs` timing data from each caption event. Chunks have no temporal metadata, so search results cannot link to a playback timestamp.

The `summarize` CLI path returns plain text with no timestamps, so timestamps are yt-dlp-only.

### Design

**Model changes**:
- Add `timed_text: Option<Vec<TimedSegment>>` to `Transcript` (`#[serde(default)]` for backwards compat)
- `TimedSegment { start_sec: f32, text: String }` - new public struct, exported as TS binding
- Add `start_sec: Option<f32>` to `ChunkDraft`, `SearchIndexChunk`, `SearchCandidate`, `SearchMatchPayload`

**Transcript extraction**:
- `parse_json3_transcript` returns `(String, Vec<TimedSegment>)` instead of `String`
- `extract_with_ytdlp` populates `timed_text` on the stored `Transcript`

**Chunking**:
- `chunk_transcript_content` gains an optional `timed_segments: Option<&[TimedSegment]>` parameter
- When timed_segments is Some: group segments by word-count target, set `start_sec` on each chunk to the `start_sec` of its first segment
- When None (summarize path): behavior unchanged, `start_sec = None`

**Storage**:
- Chunk JSON in S3 gets optional `start_sec: Option<f32>` field
- S3 Vectors metadata gets optional `start_sec` field (stored as `Number::Float`)

**API response**:
- `SearchMatchPayload.start_sec: Option<f32>` - `#[ts(optional)]`
- Frontend can use this to generate a `&t=<sec>` deep-link

### Files

- `backend/src/models.rs` - add `TimedSegment`, update `Transcript`, `SearchMatchPayload`
- `backend/src/services/transcript.rs` - update `parse_json3_transcript`, `extract_with_ytdlp`
- `backend/src/services/search.rs` - add `start_sec` to `ChunkDraft`; update `chunk_transcript_content` signature
- `backend/src/db/search.rs` - store/retrieve `start_sec` in chunk JSON and vector metadata; update `SearchIndexChunk`, `SearchCandidate`
- `backend/src/workers/search_index.rs` - pass `timed_text` from transcript into chunking call
- `backend/src/handlers/search.rs` - pass `start_sec` through grouping into `SearchMatchPayload`
- `frontend/src/lib/bindings/` - regenerated by `cargo test`

### Acceptance criteria

- Videos processed via yt-dlp fallback have `timed_text` populated in stored Transcript
- Search results for those videos carry `start_sec` in `SearchMatchPayload`
- Videos processed via summarize path have `start_sec: null` in results
- Old chunk JSONs (without `start_sec`) deserialize cleanly (`#[serde(default)]`)

---

## Feature 3: channel_id as Server-side S3 Vectors Filter

### Problem

`search_vector_candidates` (`db/search.rs:534`) over-fetches 3x from S3 Vectors when a `channel_id` filter is set and applies the filter in Rust after the fact. `channel_id` is not stored in the vector metadata.

### Design

Add `channel_id` to the S3 Vectors metadata at write time. Apply it as a server-side metadata filter in `query_vectors` (same pattern as `source_kind` filter at line 551).

`SearchMaterial` gets a `channel_id: String` field, populated in `load_search_material`. `replace_search_chunks` gains a `channel_id: &str` parameter. The search index worker passes it through from the loaded material.

Remove the 3x over-fetch multiplier for channel-scoped vector queries.

### Files

- `backend/src/db/mod.rs` - add `channel_id` to `SearchMaterial`
- `backend/src/db/search.rs` - populate `channel_id` in `load_search_material`; add param to `replace_search_chunks`; store `channel_id` in metadata; server-side filter in `search_vector_candidates`; remove 3x over-fetch
- `backend/src/workers/search_index.rs` - pass `material.channel_id` to `replace_search_chunks`

### Acceptance criteria

- Channel-scoped vector search applies the filter server-side
- No over-fetch multiplier applied for channel-scoped queries
- Existing non-scoped queries unaffected
- New chunks written after this change have `channel_id` in metadata; old chunks (without it) continue to work via client-side fallback for one release cycle, then dropped

---

## Feature 4: Neural Reranker (Ollama)

### Problem

After RRF fusion the final ranking depends on heuristics (exact phrase match, summary preference, title term coverage). A cross-encoder reranker produces true query-document relevance scores that beat these heuristics.

### Design

Ollama exposes `/api/rerank` for cross-encoder models (e.g., `bge-reranker-v2-m3`). Add a `rerank` method to `SearchService` that POSTs `{ model, query, documents: [chunk_text, ...] }` and returns re-scored ordering.

The reranker is **optional**: if no `SEARCH_RERANK_MODEL` env var is set, the existing heuristic reranking runs as before. When configured, apply it to the top-`RERANK_CANDIDATE_LIMIT` (default 30) candidates after RRF fusion in hybrid mode.

**Input to reranker**: `query + "\n\n" + chunk_text` - same pattern as embedding input but without the structured header (cross-encoders work on raw text pairs).

**Latency**: Reranking 30 candidates is a single HTTP call to a local Ollama model. Log `rerank_elapsed_ms` in the existing search log line.

### Files

- `backend/src/config.rs` - add `SEARCH_RERANK_MODEL` env var
- `backend/src/services/search.rs` - add `rerank_model: Option<String>` to `SearchService`; add `rerank_candidates` method
- `backend/src/state.rs` - wire rerank model into `SearchService` construction
- `backend/src/handlers/search.rs` - call reranker after RRF fusion; replace heuristic rerank in hybrid mode when reranker is available; keep heuristic for keyword-only mode

### Acceptance criteria

- When `SEARCH_RERANK_MODEL` is unset, behavior is identical to before
- When set, hybrid mode applies the neural reranker to top candidates
- Reranker failure (model not loaded, timeout) falls back to heuristic with a warning log
- `rerank_elapsed_ms` appears in the search log line

---

## Feature 5: HyDE for Short Queries

### Problem

Short queries (e.g., "attention mechanism", "RL reward shaping") produce low-dimensional embeddings that match poorly against 300-word chunk embeddings. The cosine similarity gap between a 2-word query vector and a paragraph vector is large.

### Design

**HyDE** (Hypothetical Document Embeddings): For queries with `<= 4` meaningful tokens and semantic search enabled, generate a short hypothetical passage (~3 sentences) via the Ollama chat model, embed the passage instead of the raw query. The passage resembles the kind of content a relevant chunk would contain, shrinking the cosine distance.

**Prompt**: Minimal - `"Write a short paragraph that directly answers the question or explains the concept: {query}"`. No system context, no conversation history. Run as a single-turn completion.

**Guard rails**:
- Only trigger when `execution_mode` is `Semantic` or `Hybrid`
- Only trigger when query has `<= 4` meaningful tokens (after stopword removal)
- If the chat LLM is unavailable or takes > 3s, fall back to raw query embedding with a debug log
- Log `hyde_elapsed_ms` and `hyde_triggered: bool` in the search log line

**LLM selection**: Use the same Ollama base URL already in AppState. Use a separate env var `SEARCH_HYDE_MODEL` (can be the same local model as chat, e.g., `qwen3:8b`). If unset, HyDE is disabled.

### Files

- `backend/src/config.rs` - add `SEARCH_HYDE_MODEL` env var
- `backend/src/services/search.rs` - add `hyde_model: Option<String>` to `SearchService`; add `generate_hyde_passage` method (single Ollama `/api/generate` call)
- `backend/src/handlers/search.rs` - conditionally replace `query` with hypothetical passage before embedding; log HyDE metrics

### Acceptance criteria

- Short queries trigger HyDE when model is configured
- Long queries (> 4 meaningful tokens) bypass HyDE
- HyDE model unavailability falls back silently
- `hyde_triggered` visible in Logfire traces

---

## Feature 6: Timed Semantic Chunking for Transcripts

### Problem

Current transcript chunking splits by paragraph breaks then word-count windows. When timestamps are available (yt-dlp path), each caption event is a natural semantic unit. Grouping events by word-count target (rather than word-splitting arbitrary text) preserves sentence integrity and produces cleaner chunk boundaries.

### Design

After Feature 2 lands (timestamps), `chunk_transcript_content` can take `timed_segments` and group them into chunks by accumulated word count, respecting segment boundaries. Each chunk's `start_sec` is the `start_sec` of its first segment.

When `timed_segments` is None (summarize path), the current paragraph-then-word-count chunking runs unchanged.

Overlap: for timed chunking, re-include the last N words of the previous chunk as a new "overlap segment" with the previous chunk's `start_sec`. This maintains the overlap semantics without breaking segment boundaries.

### Files

- `backend/src/services/search.rs` - new `chunk_transcript_timed` function; called from `chunk_transcript_content` when `timed_segments.is_some()`

### Acceptance criteria

- Timed chunks respect segment boundaries (no mid-sentence splits)
- `start_sec` on each chunk matches the first segment in that chunk
- Untimed transcripts produce identical output to current behavior
- Unit tests cover: single-segment chunks, multi-segment grouping, overlap behavior
