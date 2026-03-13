# Tasks: Semantic Search

## Current State
Local-first semantic search is implemented with Ollama embeddings, Turso-backed projection tables, a background indexing worker, and a workspace search UI. The runtime schema is now aligned to the final Turso-first design: `search_sources` is the parent per-source state table with integer ids and source generations, `search_chunks` is the only persisted chunk-content table, and FTS is maintained as an external-content projection over `search_chunks` instead of a second wide text table. The live query path now defaults to plain `fts_only` unless `SEARCH_SEMANTIC_ENABLED=true` is set, in which case it uses `hybrid_exact` (`FTS5 shortlist + exact cosine rerank`) whenever embeddings are available and no ANN index exists, and upgrades to `hybrid_ann` only when `idx_search_chunks_embedding` is actually present. There is now also a search-only rebuild path via `POST /api/search/rebuild` that drops and recreates only the derived search projection, leaving canonical content untouched, and ANN auto-creation is opt-in via `SEARCH_AUTO_CREATE_VECTOR_INDEX=false` by default. The workspace header now keeps the subtle search-coverage hint visible whenever indexed-source totals exist, including the fully indexed `100% indexed` state, with a raw-count fallback below 1%. In the indexing pipeline, summaries are now explicitly prioritized ahead of transcripts for both backfill discovery and pending-source processing so summary searchability is no longer blocked behind a large transcript backlog. Startup bootstrap responses now also include `search_status`, so the initial workspace render can show indexing progress without waiting for the separate status poll, and total-source counts now come from canonical `videos` readiness flags instead of large transcript/summary text joins to keep the status path lightweight. Current local verification passed with `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release`, `cargo fmt --check`, full frontend `bun test`, `bun run check`, `bun run build`, and `bun run format:check`. Targeted backend/frontend regressions for bootstrap search status also passed. Manual live verification against an alternate local release port was inconclusive in this environment because the standalone backend process did not reliably stay bound long enough to replay `/api/workspace/bootstrap`, so the remaining work is still a live rebuild/manual seeded-data verification pass against the real Turso/Ollama environment.

## Steps
- [x] Confirm the remaining product choices for the local-first release: acceptable indexing lag, default search scope, and multilingual expectations.
- [x] Add Turso schema changes for search source state, chunk storage, FTS5 indexing, and the vector index.
- [x] Add backend configuration for local Ollama embeddings and a typed search service interface.
- [x] Implement transcript and summary chunkers plus content hashing for idempotent indexing.
- [x] Hook transcript write, summary write, regenerate, manual edit, and delete flows into lightweight search job signaling and cleanup without doing embedding work inline.
- [x] Add an isolated background indexing worker plus backfill and reconciliation flows for already-ingested content and missed indexing events.
- [x] Implement the hybrid retrieval query path and the `/api/search` response contract.
- [x] Add the frontend search bar, source filters, result list, and deep-link behavior into the existing Svelte workspace.
- [x] Add backend tests for chunking, indexing sync, ranking, and API behavior.
- [x] Add frontend tests for search state, API calls, and result rendering.
- [x] Align the runtime schema and retrieval path to the final Turso-first architecture (`search_sources` parent rows, `search_chunks` external-content FTS, exact hybrid baseline, optional ANN).
- [x] Add a search-only rebuild path that preserves canonical content while resetting the derived search projection.
- [ ] Run a live search-projection rebuild and repopulation pass against the configured Turso/Ollama environment.
- [ ] Run format, lint, tests, and a manual search verification against representative seeded data before proposing the stack.

## Decisions Made During Implementation
- Turso remains the only database and vector store.
- Default retrieval design is hybrid (`FTS5 + vector`) rather than vector-only.
- The first implementation is local-only and uses Ollama embeddings.
- Embedding model selection is config-only via `OLLAMA_EMBEDDING_MODEL`; there is no hardcoded application default.
- Default synchronization design is app-managed async indexing plus reconciliation, not trigger-only or inline embedding at content-write time.
- Existing ready transcripts and summaries are now discovered through a bounded backfill query that seeds missing `search_sources` rows, while reconcile is reserved for failed or stale indexed rows.
- Search backfill, reconcile, and indexing now log batch-level summaries with consistent structured counts; indexing summaries also include the embedding model and embedded chunk totals.
- Search queries only read chunks indexed with the currently configured embedding model, and reconcile marks rows pending again when the stored model and runtime model diverge.
- The summary chunker keeps one broad full-summary chunk plus section chunks, and the broad chunk is no longer truncated to the 420-character display snippet limit before embedding.
- The current UI keeps search in the workspace header, positions the input just left of the workspace tabs, and renders the source filter plus results in a floating popup instead of replacing the video list.
- The vector index is intentionally excluded from startup migrations because remote Turso writes become prohibitively slow once `idx_search_chunks_embedding` exists; it must be created as a separate operational step after bulk indexing.
- Search now treats a missing vector index or unreachable embedding model as `fts_only` retrieval instead of failing the request or embedding the query unnecessarily.
- Migration DDL now runs statement-by-statement to reduce the risk of remote Turso stalls on a large multi-statement `execute_batch`.
- The derived search projection no longer tries to preserve backwards compatibility. If the projection schema changes, startup drops and recreates `search_chunks` plus `search_chunks_fts`, then requeues `search_sources` from canonical content.
- `search_sources` now owns the source identity and indexing lifecycle with an integer `id`, `(video_id, source_kind)` uniqueness, `content_hash`, and `source_generation`; `search_chunks` references `search_source_id` instead of repeating source identity in every chunk row.
- `mark_search_source_pending()` no longer deletes current chunk rows immediately. Pending rows become invisible through the `search_sources` join, and a bounded prune pass reclaims stale chunk/FTS rows asynchronously to reduce write amplification on remote Turso.
- Worker throughput was increased conservatively after reducing write churn, but the vector index is still excluded from the remote bulk-indexing phase.
- Architecture checkpoint before the destructive reset: the end-state design should use `search_sources` as the parent source-state table, `search_chunks` as the only persisted chunk-content table, external-content FTS over `search_chunks`, exact vector reranking as the default hybrid strategy, and ANN only as an optional serving acceleration after the backlog reaches zero.
- Runtime search now reports `fts_only`, `hybrid_exact`, or `hybrid_ann` instead of a binary hybrid flag so status surfaces match the actual query strategy in use.
- `POST /api/search/rebuild` now resets only `search_sources`, `search_chunks`, and `search_chunks_fts`; canonical tables are intentionally left intact so reindexing does not require a full DB wipe.
- Automatic ANN index creation is now disabled by default and can be explicitly enabled with `SEARCH_AUTO_CREATE_VECTOR_INDEX=true` after a backlog-free rebuild if query latency justifies it.
- The workspace search bar keeps the subtle coverage hint visible whenever `total_sources > 0`, including `100% indexed`, and falls back to raw counts when rounded percentages would hide sub-1% progress.
- Semantic search is now explicitly opt-in via `SEARCH_SEMANTIC_ENABLED=true`; the default runtime behavior is plain FTS indexing and query serving, which keeps production search working without any embedding model or vector index.
- Search indexing now prioritizes summary sources before transcript sources when discovering missing work and when claiming pending work, but query-time search result ordering is unchanged.
- Workspace bootstrap now includes `search_status` so the indexing hint can render on first load, and the total-source count intentionally uses `videos.transcript_status` plus `videos.summary_status` rather than scanning transcript/summary text bodies to avoid stalling remote startup requests.
