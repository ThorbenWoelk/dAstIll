# Search Retrieval Upgrade (Hybrid RAG)

**Linear:** N/A

## Problem

Workspace search combines a keyword leg and a dense-vector leg (RRF fusion, then heuristic reranking). Several bottlenecks and quality gaps limit scale and UX:

- **Keyword leg:** `search_fts_candidates` lists every object under `search-chunks/`, fetches each JSON, and scores with substring token overlap. That is effectively O(n) over the corpus with no BM25-style term weighting or field norms (`backend/src/db/search.rs`, FTS path).
- **Vector leg:** When `channel_id` is set, `search_vector_candidates` over-fetches (`top_k = limit * 3`) and filters by channel after loading videos, because metadata filtering is only applied for `source_kind` today (`backend/src/db/search.rs`).
- **Transcript chunks:** `ChunkDraft` and transcript chunking carry text only; there is no `start_sec` / `end_sec` for transcript-derived chunks, so results cannot deep-link to playback time (`backend/src/services/search.rs`).

Secondary gaps called out in review: no neural reranker after fusion, no HyDE for short queries, hardcoded hybrid overfetch ratios, limited retrieval-quality logging, word-count-only transcript chunking, and fixed 512-d embedding dimensions tied to the current Ollama setup.

## Goal

Deliver measurably better **recall and precision** for hybrid search, **faster** keyword and channel-scoped vector retrieval at growing corpus size, and **timestamp-aware** transcript hits so the UI can jump to the right moment in a video.

Success is verifiable by: (1) keyword path no longer full-scans every chunk JSON on each request (or equivalent documented mitigation), (2) vector queries with `channel_id` use server-side metadata filter where the platform allows, (3) transcript chunks persist and expose time bounds where the source transcript supports it, (4) remaining items phased without blocking the above.

## Requirements

### R1 - BM25-style keyword retrieval (in-process)

- Replace or supersede the current full-list fetch + substring match scoring for the keyword leg with an in-process index suitable for BM25 (e.g. Tantivy) built from the same corpus as `search-chunks/`.
- Keyword queries must return ranked candidates without O(corpus) object reads per request in the steady state (initial index build / explicit rebuild may still scan objects).
- Preserve existing filters: `source_kind`, `channel_id`, and empty-token behavior.

### R2 - `channel_id` as vector metadata filter

- Store `channel_id` in S3 Vectors document metadata for indexed chunks (alongside existing fields such as `video_id`, `source_kind`).
- When the search API is called with `channel_id`, apply a **server-side** filter on the vector query so `top_k` targets the requested scope without relying on 3x over-fetch + Rust-side filtering alone.
- Re-index or backfill strategy must be defined so existing deployments gain the metadata without silent wrong results.

### R3 - Transcript chunk timestamps

- Extend chunking and persisted chunk payloads so transcript chunks can carry **optional** `start_sec` / `end_sec` (or equivalent) when the source transcript includes parseable timing (e.g. WebVTT / SRT-style cues).
- Summary chunks may remain without timestamps.
- API payloads exposed to the frontend for search matches include timing when present so the client can implement seek / deep links.

### R4 - Neural reranker (optional path)

- After RRF (or equivalent fusion), re-order the top-K fused candidates using a cross-encoder or dedicated reranker model.
- Support a **local** path (e.g. Ollama-compatible reranker) consistent with existing embedding configuration; optional cloud provider is non-blocking if out of scope for the first implementation pass.

### R5 - HyDE for short / sparse queries

- For queries below a configurable token threshold, optionally expand the query embedding step using a hypothetical document produced via existing LLM/chat infrastructure, then run vector retrieval with that embedding.
- Must be safe to disable via config and must not block search if the LLM step fails.

### R6 - Dynamic hybrid overfetch

- Replace or parameterize fixed `fts_candidate_limit` / `semantic_candidate_limit` multipliers (`handlers/search.rs`) using signals already available (e.g. `SearchStatusPayload` / corpus estimates) so small and large libraries do not share one hardcoded policy.

### R7 - Retrieval observability

- Log structured signals per request such as top fused scores (e.g. top-3 RRF scores) and mode (keyword / semantic / hybrid) to assess confidence and tune reranking.

### R8 - Semantic transcript chunking (later phase)

- Improve transcript splitting beyond pure word-count windows: at minimum sentence-boundary-aware grouping; optionally semantic boundary detection if justified by cost and complexity.

### R9 - Embedding dimensions (future)

- Document and optionally implement a path to higher-dimensional embeddings via a configurable provider; any change implies **full re-embedding** of the index.

## Non-Goals

- Replacing S3 object storage for chunk JSON or moving the primary ANN index to a different vendor in this initiative (unless a task explicitly scopes it).
- Perfect timestamp alignment when the stored transcript has no timing data (plain text only).
- Training custom embedding or reranker models.

## Design Considerations

- **Tantivy vs external search:** In-process BM25 matches the current Rust backend and avoids new infrastructure; rebuilding the index at startup or on a schedule trades memory and boot time for query latency. Document memory bounds and rebuild triggers.
- **Metadata schema:** S3 Vectors filterable fields must stay consistent between index worker writes and query filters; migrations need a version or backfill job.
- **Timestamps:** Parsing should be defensive (malformed cues skipped); store only validated ranges.
- **Reranker + HyDE:** Add latency and failure budgets; both should degrade gracefully to current fusion + heuristic behavior when off or failing.
- **Frontend:** TypeScript bindings (`ts_rs`) must stay in sync when search payloads gain new optional fields.

## Open Questions

- Exact Tantivy schema (fields for title vs chunk body vs channel) and whether to index video title in the keyword leg for free-text queries.
- Whether reranker runs for keyword-only mode or only hybrid / semantic.
- Preferred cue format for transcripts in storage (VTT vs raw) to lock parsing behavior.
