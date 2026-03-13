# Semantic Search

**Linear:** n/a

## Problem

dAstIll can show channels, transcripts, and AI summaries, but it cannot answer a simple retrieval need: "show me every video that talked about this concept." As the app ingests more subscribed-channel content, browsing channel-by-channel stops scaling. Using a separate vector store would also introduce freshness drift from the Turso-backed canonical data model, especially because transcripts and summaries can be generated automatically and edited manually.

## Goal

Add a global search experience that accepts free-text terms or full-sentence queries and returns relevant video summary and transcript matches from the same Turso database, with automatic indexing for newly ingested or edited content and retrieval quality that handles both exact keywords and semantic paraphrases. The first shipped version is local-only and uses Ollama for embeddings.

## Requirements

- Add a global search UI entry point in the existing Svelte workspace that accepts free-text input ranging from short keywords to full-sentence queries.
- Return a ranked result list that surfaces matches from summaries and transcripts, with each result showing at minimum video title, channel, published date, source kind, and a relevant snippet.
- Let users open a result directly in the existing video/content view with the relevant mode already selected.
- Use Turso/libSQL native vector search as the only vector database. Do not introduce a separate vector store or search backend.
- Scope the first implementation to local/self-hosted environments only. Hosted production deployments may leave semantic indexing disabled.
- Keep search state automatically aligned with the canonical Turso data model. New transcripts, summaries, manual edits, regenerations, and deletions must trigger reindexing or cleanup automatically.
- Preserve the existing ingestion flow, but isolate semantic indexing from user-facing writes. Transcript and summary write paths must only mark or enqueue search work; chunking, embedding, and index refresh must run asynchronously in a separate search-index pipeline.
- Index transcript content as soon as transcript text becomes ready, and index summary content as soon as summary text becomes ready. A video must be searchable even if only one of those two sources exists.
- Support hybrid retrieval so that both exact-term queries and paraphrased/natural-language queries work well.
- Support source filters at minimum `all`, `summaries`, and `transcripts`.
- Rank results at chunk level but present them grouped by video so users can quickly decide what to open.
- Provide an initial backfill/reindex path for transcripts and summaries that already exist in Turso before the feature ships.
- Provide observability for indexing lag and failures in logs and at least one queryable status surface.
- Cover indexing logic and search API behavior with backend tests, and cover frontend search state plus API integration with frontend tests.

## Non-Goals

- Conversational QA or answer synthesis across multiple videos
- Timestamp-level seek inside videos
- A second external vector database
- LLM-generated answers in the search UI
- Trigger-only synchronization as the default indexing architecture

## Design Considerations

- Keep the canonical content hierarchy unchanged: `channels -> videos -> {transcripts, summaries, video_info, highlights}` remains the source of truth. Search is a derived projection over that hierarchy and must never become the canonical owner of titles, channel names, or publish dates.
- Use Turso native vectors and FTS5 together, but separate the *retrieval strategy* from the *serving acceleration strategy*. Turso documents native vector columns, vector functions, vector indexes, and `vector_top_k`, and also ships FTS5 as a built-in SQLite extension. For this app, the baseline hybrid strategy should be `FTS5 shortlist + exact vector rerank` over stored embeddings. ANN should be optional and added later only if query latency, not indexing throughput, becomes the bottleneck.
- Local embedding model selection is config-only via `OLLAMA_EMBEDDING_MODEL`, with a fixed output size configured explicitly at startup. Ollama's `/api/embed` supports a `dimensions` parameter for compatible models, and local environments can choose `embeddinggemma`, `qwen3-embedding`, or another supported Ollama embedding model without code changes.
- Default vector metric: cosine. Turso's native vector examples use cosine distance, and cosine is the right default for normalized text embeddings.
- Do not embed entire transcripts as single vectors. Transcript retrieval quality will be too coarse once videos become long. Transcript search needs chunk-level indexing.
- Summary chunking strategy: always create one video-level summary chunk for broad recall. When a summary contains markdown sections or exceeds about 400 tokens, additionally create heading-aware summary chunks capped around 800 tokens. Preserve heading titles as metadata and strip markdown formatting from the embedded body text.
- Transcript chunking strategy: build transcript search text from `raw_text` when available; otherwise strip markdown from `formatted_markdown`. Create paragraph/sentence-aware chunks targeting about 800 tokens with about 200 tokens of overlap. Prefer natural paragraph boundaries and fall back to sentence windows only when needed.
- Chunk-size rationale: the initial local-only implementation should still target roughly 800-token transcript chunks, but use a smaller overlap around 200 tokens to reduce duplicate local embeddings and index growth. Treat that overlap value as a tuning default that must be validated with relevance tests.
- Embedding payload strategy: include lightweight metadata such as video title, channel name, and section label in the text sent for embedding, but store clean display text separately for snippets and UI rendering.
- Synchronization design: add search projection tables inside the same Turso database. On every transcript or summary write path, compute a content hash and mark the corresponding search source as `pending`. That write path must stay lightweight and must not wait for chunking or embedding calls. A separate indexing worker claims pending sources, computes chunks and embeddings, writes a new source generation, and marks the source `ready`. Deletes and regenerations must clear or replace stale chunk rows automatically. A reconciliation scan must periodically compare current content hashes and stored embedding model names to the runtime configuration so missed events and model switches self-heal.
- Operational model: treat semantic search as an internal projection pipeline fed by canonical Turso content tables. This keeps freshness tied to the main data model while isolating slower embedding work, rate limits, retries, and provider failures from the transcript/summary ingestion latency budget.
- Deployment model for v1: run semantic indexing only when a local Ollama embedding model is configured and reachable. If embeddings are unavailable, the search UI should surface that semantic indexing is unavailable rather than silently pretending semantic search exists.
- Use Turso triggers only for local FTS projection maintenance, not for canonical application logic. Current Turso docs describe full trigger support, and this is the right place to use them: `search_chunks` stays the only chunk-content store, while FTS rows are maintained automatically from it. Canonical ingest, pending-state transitions, and reconciliation remain app-managed.
- Suggested schema:
  - `search_sources(id INTEGER PRIMARY KEY, video_id TEXT NOT NULL, source_kind TEXT NOT NULL, content_hash TEXT NOT NULL, source_generation INTEGER NOT NULL DEFAULT 0, embedding_model TEXT, index_status TEXT NOT NULL, last_indexed_at TEXT, last_error TEXT, UNIQUE(video_id, source_kind), FOREIGN KEY(video_id) REFERENCES videos(id))`
  - `search_chunks(id INTEGER PRIMARY KEY, search_source_id INTEGER NOT NULL, source_generation INTEGER NOT NULL, chunk_index INTEGER NOT NULL, section_title TEXT, chunk_text TEXT NOT NULL, token_count INTEGER NOT NULL, embedding F32_BLOB(512), UNIQUE(search_source_id, source_generation, chunk_index), FOREIGN KEY(search_source_id) REFERENCES search_sources(id))`
  - `search_chunks_fts` as an external-content FTS5 table over `search_chunks(section_title, chunk_text)` with insert/update/delete triggers on `search_chunks`
  - optional `idx_search_chunks_embedding` on `libsql_vector_idx(embedding, 'metric=cosine')`, created only after a full rebuild or once the backlog reaches zero
- Query pipeline:
  - baseline: normalize the query, generate one embedding, run FTS5 keyword search for a candidate shortlist, rerank that shortlist with exact cosine distance against `search_chunks.embedding`, group matches by `video_id`, and return top videos with supporting chunk snippets and source labels
  - optional acceleration: when ANN is enabled, add `vector_top_k(...)` as a second candidate source and fuse ANN plus FTS with reciprocal rank fusion
- Status/reporting model: expose at least `pending`, `indexing`, `ready`, `failed`, `total_sources`, and retrieval mode. Retrieval mode should distinguish `fts_only`, `hybrid_exact`, and `hybrid_ann` so the UI and logs reflect the true backend state.
- API surface: `GET /api/search?q=...&source=all|summary|transcript&limit=...&channel_id=...` returning a video-centric payload with nested matches instead of only a flat chunk list.
- Frontend UX: place the search bar in the main workspace header so it is global rather than tied to one channel panel; debounce typing while still supporting explicit submission on Enter; show loading, empty, and error states; and open clicked results in the relevant existing content mode.

## Open Questions

- Is a few-seconds indexing delay acceptable for newly ingested or manually edited content, or must searchability be effectively immediate after write completion?
- Should v1 search default to the currently selected channel when a channel is active, or always search globally first?
- For future hosted deployments, should semantic search remain local/self-hosted only, or should the app add a managed embedding provider behind a separate deployment mode?
- Is multilingual retrieval a first-class requirement, or can v1 optimize primarily for the app's dominant content language?

## References

- Research checked on March 12, 2026.
- Turso AI & Embeddings docs for native vector columns, `F32_BLOB`, `libsql_vector_idx`, and `vector_top_k`.
- Turso SQLite Extensions docs for built-in FTS5 and the recommendation to use native libSQL vectors instead of vector extensions.
- Turso Triggers docs for current trigger support and cautions around sync-heavy workloads.
- Ollama embeddings capability docs for `/api/embed`, similarity usage, and model consistency requirements.
- Ollama library pages for current local embedding model options.
