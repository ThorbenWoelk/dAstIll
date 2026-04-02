# Search Indexing

<script setup>
const queryPathDiagram = String.raw`
flowchart LR
  query[User query]
  tokenize[Tokenize + remove stopwords]
  hyde[Optional HyDE passage]
  embed[Embed query or HyDE passage]
  fts[BM25 libSQL FTS5 leg]
  vectors[Vector retrieval leg]
  rrf[RRF fusion]
  rerank[Optional reranker]
  results[Search results + chat sources]

  query --> tokenize
  tokenize --> fts
  tokenize --> hyde
  tokenize --> embed
  hyde --> embed
  embed --> vectors
  fts --> rrf
  vectors --> rrf
  rrf --> rerank
  rerank --> results
`;

const indexMaintenanceDiagram = String.raw`
flowchart LR
  canonical[Ready transcript or summary]
  pending[search_sources pending]
  claim[Search worker claims source]
  chunk[Chunk content]
  embed[Optional embedding batches]
  store[Write search_chunks + source state]
  fts[Upsert libSQL FTS5 source]
  vectors[S3 Vectors]
  retrieval[Keyword + hybrid retrieval]

  canonical --> pending
  pending --> claim
  claim --> chunk
  chunk --> embed
  chunk --> store
  embed --> store
  store --> fts
  store --> vectors
  fts --> retrieval
  vectors --> retrieval
`;
</script>

## Overview

Search is built on three complementary layers that run in parallel and can be combined
at query time:

1. **BM25 keyword search (libSQL FTS5)** - keyword retrieval with weighted title/section
   fields and an FTS pre-ranker for title/phrase signal boosting
2. **Vector search (S3 Vectors)** - semantic similarity via dense embeddings
3. **RRF fusion** - merges ranks from both legs; optionally fed to a cross-encoder reranker

Each layer degrades independently without breaking the others.

<MermaidDiagram
  caption="Search query path: keyword and semantic legs run independently, then merge before the final result set is returned to workspace search or chat."
  :chart="queryPathDiagram"
/>

---

## Storage Backend

| Store          | Role                                                                             |
| -------------- | -------------------------------------------------------------------------------- |
| S3 data bucket | Canonical chunk JSON objects under `search-chunks/`                              |
| Turso / libSQL | Durable keyword index primary plus local embedded replica file for runtime reads |
| S3 Vectors     | Dense embeddings for ANN retrieval, keyed by chunk ID                            |

S3 remains the rebuild source of truth for canonical chunk content. The keyword index lives in
libSQL/Turso and is bootstrapped from the stored projection when the runtime index is empty.

<MermaidDiagram
  caption="Index maintenance flow: canonical content becomes pending search sources, then the search worker chunks, embeds, stores, and syncs the libSQL FTS5 keyword index."
  :chart="indexMaintenanceDiagram"
/>

---

## Keyword Index

### Schema

| Field           | Tokenizer   | Role                                                    |
| --------------- | ----------- | ------------------------------------------------------- |
| `chunk_id`      | stored only | Unique chunk identifier                                 |
| `video_id`      | stored only | Parent video                                            |
| `channel_id`    | stored only | Parent channel (used for post-filter)                   |
| `source_kind`   | stored only | `transcript` or `summary`                               |
| `source_key`    | `raw`       | Composite `{video_id}_{source_kind}` for exact deletion |
| `chunk_text`    | `en_stem`   | Primary searchable text (BM25 scoring)                  |
| `section_title` | `en_stem`   | Summary section heading (optional, also scored)         |
| `video_title`   | `en_stem`   | Video title (also scored)                               |
| `channel_name`  | stored only | Human-readable channel name                             |
| `published_at`  | stored only | ISO 8601 publication date for sort ordering             |
| `start_sec`     | stored only | Caption start timestamp in seconds (transcript only)    |

The keyword index stores the same retrieval metadata alongside the FTS5 document so a
single query can return chunk text, title data, source kind, and optional timestamps
without a second metadata lookup.

### Startup Hydration

`populate_fts_index_from_store` runs once at startup only when the runtime keyword index
is empty:

1. Lists all objects under the `search-chunks/` S3 prefix
2. Fetches them concurrently (up to 32 in parallel)
3. Groups chunks by `(video_id, source_kind)`
4. Loads video and channel metadata for each group
5. Calls `fts.upsert_source` for each group

When a Turso-backed embedded replica already has indexed rows, startup skips this replay
and uses the existing local replica immediately.

### Live Sync

The search index worker keeps the libSQL/Turso keyword index in sync after every write:

- After writing chunks to S3, `fts.upsert_source` replaces keyword-search rows for that
  `(video_id, source_kind)` pair
- After clearing a source (content removed or empty draft list), `fts.delete_source`
  removes the corresponding rows

The `upsert_source` path always deletes the old documents for the source key before
inserting new ones, so re-indexing a video does not accumulate duplicate chunks.

---

## Search Worker Phases

The search worker is a background loop with four recurring responsibilities.

### Backfill

Discovers canonical transcript and summary content that should already be searchable but
has no `search_sources` row yet. Runs at the start of each loop iteration.

### Index Pending Sources

Claims pending rows, loads canonical content, chunks it, optionally embeds it, and
writes the derived chunk projection. After each successful write, the keyword index is
updated synchronously before the loop moves on.

### Reconcile

Finds stale indexed rows and requeues them when:

- content hashes changed
- indexing previously failed
- the stored embedding model no longer matches the runtime model

Runs on a longer cadence than the pending-source pass to avoid constant churn.

### Prune

Removes stale chunk rows that are no longer referenced by a ready source generation.

## Summary Priority

Indexing intentionally prioritizes summaries before transcripts when:

- discovering missing work
- claiming pending work
- selecting reconciliation work

This keeps summary searchability from being starved behind a large transcript backlog.

---

## Chunking Strategy

### Chunking Parameters

| Constant                   | Value | Description                                                   |
| -------------------------- | ----- | ------------------------------------------------------------- |
| `TRANSCRIPT_TARGET_WORDS`  | 300   | Target words per transcript chunk                             |
| `TRANSCRIPT_OVERLAP_WORDS` | 40    | Overlap words between consecutive transcript chunks           |
| `SUMMARY_TARGET_WORDS`     | 300   | Target words per summary section chunk                        |
| `EMBEDDING_DIMENSIONS`     | 512   | Vector dimensions for the common embeddinggemma configuration |
| `EMBED_BATCH_SIZE`         | 8     | Chunks per embedding API request                              |

### Transcript Chunking

Transcript chunking selects a strategy based on available input:

**Timed chunking** (preferred): used when caption segments with timestamps are available
(the `yt-dlp json3` fallback path). Segments are grouped by word count. Each chunk
records `start_sec` from the first segment it contains - this timestamp flows all the
way to search results, enabling deep-link navigation to the exact position in the source
video. Overlap is applied the same way as paragraph chunking: the last 40 words of the
previous chunk prefix the next chunk.

**Paragraph chunking** (default): used when only plain text is available. The algorithm:

1. Splits on blank lines to identify natural paragraph boundaries
2. Accumulates paragraphs until the next one would exceed `TRANSCRIPT_TARGET_WORDS` (300)
3. Splits paragraphs that alone exceed the target into word-based sub-chunks
4. Carries the last `TRANSCRIPT_OVERLAP_WORDS` (40) words forward as a prefix for the
   next chunk to bridge query terms that span boundaries

Paragraph chunking sets `start_sec` to null.

### Summary Chunking

Summary chunking produces two tiers per document:

1. **Full-document chunk** (`is_full_document: true`): the entire normalized summary text
   as one chunk. Never split. Serves broad or full-context queries and guarantees the
   complete summary is always retrievable regardless of section structure.

2. **Section chunks** (`is_full_document: false`): one chunk per `## ` heading section.
   Sections under `SUMMARY_TARGET_WORDS` (300) become a single chunk with the section
   title preserved. Longer sections are split into word-based sub-chunks. Each chunk
   carries its `section_title` for display and optional filtering.

### Chunk Content Normalization

Before chunking, text is normalized by a shared `normalize_source_text` function:

- strips markdown heading prefixes (`#`, `##`, etc.) from line starts
- strips list markers (`-`, `*`, `1.`, `2)`)
- collapses blank lines and excess whitespace into single spaces
- trims leading/trailing whitespace

This produces clean searchable text without formatting artifacts.

---

## Query Path

At query time the backend runs a multi-stage pipeline. Below is the full decision tree.

### 1. Query tokenization

The raw query is tokenized into meaningful keyword terms using a stopword-aware
tokenizer. Common question words and filler terms are removed. The remaining terms are
used for FTS and for snippet extraction centering.

Terms are also deduplicated and capped at 4 after deduplication:

```
"rust rust tokio axum libsql semantic search" -> ["rust", "tokio", "axum", "libsql"]
"what is the best db in town"                 -> ["db", "town"]
```

### 2. HyDE (optional)

Triggered when all of the following are true:

- `SEARCH_HYDE_MODEL` is configured
- semantic search is enabled for the request
- the query contains **4 or fewer meaningful tokens** after stopword removal

When triggered, the backend calls Ollama `/api/generate` (30 s timeout) with this prompt:

```
Write a concise 2-3 sentence passage that directly answers: "<query>".
Be specific. Output only the passage, nothing else.
```

The generated passage is used as the embedding input instead of the raw query. The
original raw query is still used for FTS, FTS pre-ranking, and keyword snippet
extraction.

Falls back to embedding the raw query on any failure (timeout, empty passage).

### 3. FTS leg

BM25 search against the libSQL FTS5 index. The FTS match query targets three fields:
`chunk_text`, `video_title`, and `section_title`. Channel and source-kind filters are
applied in SQL before the final ranked result set is returned.

The candidate fetch limit varies by execution mode:

| Execution mode | FTS candidate limit            |
| -------------- | ------------------------------ |
| `hybrid`       | `limit * 8`, clamped to 10-100 |
| `keyword`      | `limit * 2`, clamped to 10-50  |
| `semantic`     | 0 (FTS leg skipped)            |

Results are post-processed by `extract_keyword_snippet` to center the display snippet
around the earliest matching query token. If the chunk text is under 420 characters,
it is used as-is; otherwise a 420-character window centered on the match is returned,
prefixed/suffixed with `...`.

After BM25 retrieval, a **FTS pre-ranker** re-sorts the candidates before they reach
RRF fusion. It applies the following sort key in order:

1. Exact phrase match in chunk text, video title, or section title (highest priority)
2. Source kind: summaries before transcripts
3. All query terms present in video title
4. Number of query terms present in video title
5. Original BM25 rank (tiebreaker)

This rewards candidates where the query as a phrase or the full query term set appears
in a title, without discarding BM25 scores for less obvious matches.

### 4. Semantic leg (optional)

Requires semantic search enabled and an embedding model configured. Embeds the HyDE
passage or raw query using Ollama `/api/embed` (90 s timeout, up to 8 inputs per
batch request).

Two retrieval paths:

| Retrieval mode | Mechanism                                           | Candidate limit             |
| -------------- | --------------------------------------------------- | --------------------------- |
| `hybrid_ann`   | ANN query via S3 Vectors                            | `limit * 8`, clamped 10-100 |
| `hybrid_exact` | Exact dot-product scan via S3 (pre-ANN-index state) | `limit * 4`, clamped 10-50  |

Both paths accept server-side metadata filters for `source_kind` and `channel_id`
so only relevant chunks are scored.

Special case for `source=all` + `hybrid_exact`: exact scan is scoped to summaries
only (to keep latency bounded). The ANN path handles all source kinds.

### 5. RRF fusion

Reciprocal Rank Fusion combines the FTS and semantic ranked lists into a single score:

```
score(chunk) = sum over each list L where chunk appears:
               1 / (K + rank_in_L)
```

The constant `K = 60.0`. Chunks seen by both retrievers accumulate contributions from
both lists, naturally surfacing results with cross-retriever agreement. Chunks present
in only one list still get a contribution. The fused list is sorted descending by score.

### 6. Neural reranker (optional)

Applied when `SEARCH_RERANK_MODEL` is configured and execution mode is `Hybrid` and
both candidate lists are non-empty. Pipeline:

1. RRF merges the vector and FTS candidates into a flat ordered list
   (`collect_rrf_candidates`)
2. The top 50 chunks (by RRF score) plus the original query are posted to Ollama
   `/api/rerank` (30 s timeout)
3. Results are sorted by `relevance_score` descending
4. The reranked list is passed to `group_ranked_candidates` for video grouping

Falls back to the plain RRF ordering if the reranker call fails or returns no results.

### 7. Result grouping

Results are grouped by `video_id`. Each group carries:

- the video title, channel, and `published_at` for display
- up to one match per source kind (transcript and/or summary), taking the highest-scored
  match for each

Video groups are sorted by the best rank among their contained chunks. Within a group,
matches are sorted by score descending. The result set is truncated to `limit` (default
8, max 25) video groups.

---

## Execution Mode Matrix

The `mode` query parameter selects the execution strategy:

| Mode       | FTS | Semantic | Fusion | Reranker      |
| ---------- | --- | -------- | ------ | ------------- |
| `keyword`  | Yes | No       | No     | No            |
| `semantic` | No  | Yes      | No     | No            |
| `hybrid`   | Yes | Yes      | RRF    | If configured |

If semantic search is unconfigured or the embedding call fails, `hybrid` degrades to
FTS-only for that request. If either candidate list is empty, the other list is used
directly without fusion.

---

## Retrieval Modes (Status Surface)

The runtime status reports which retrieval mode is currently active for hybrid search:

| Status mode    | Condition                                                  |
| -------------- | ---------------------------------------------------------- |
| `fts_only`     | Semantic search disabled, or no embedding model configured |
| `hybrid_exact` | Semantic enabled; S3 Vectors ANN index not yet ready       |
| `hybrid_ann`   | Semantic enabled; S3 Vectors ANN index is ready            |

The neural reranker is transparent to this status - it activates within `hybrid_exact`
and `hybrid_ann` when configured, but the reported mode does not change.

---

## Semantic Enablement

The search service only generates embeddings when semantic search is enabled.

If semantic search is disabled:

- search sources are still chunked and indexed in libSQL FTS5
- FTS still works
- `embedded_chunk_count` remains `0`
- `vector_index_ready` remains `false`

`SEARCH_SEMANTIC_ENABLED` overrides either direction. Local debug runs default semantic
on; release builds default semantic off.

---

## Embedding Model

The embedding service reads the configured model name from `OLLAMA_EMBEDDING_MODEL`.
There is no separate hard-coded fallback in the search service. The embedding service:

- calls Ollama's `/api/embed` endpoint
- batches embedding requests (up to 8 chunks per request)
- validates that returned dimensions match the configured model
- checks model availability at startup via Ollama `/api/tags`

### Embedding Input Format

Chunks are enriched with metadata before embedding to anchor the vector in topic space:

```text
Video: <video_title>
Channel: <channel_name>
Source: transcript|summary
Section: <section_title>  (omitted when empty)

<chunk_text>
```

This prefix moves the chunk vector toward the video/channel topic cluster and improves
recall for queries that reference the content area rather than verbatim phrases.

---

## API

### `GET /api/search`

| Parameter    | Type    | Default  | Description                          |
| ------------ | ------- | -------- | ------------------------------------ |
| `q`          | string  | required | Search query                         |
| `source`     | enum    | `all`    | `all`, `transcript`, `summary`       |
| `limit`      | integer | `8`      | Max video groups returned (1-25)     |
| `channel_id` | string  | -        | Restrict results to a single channel |
| `mode`       | enum    | `hybrid` | `keyword`, `semantic`, `hybrid`      |

Returns `SearchResponsePayload` with `query`, `source`, and a list of
`SearchVideoResultPayload` objects. Each video result includes the video title,
channel, publication date, and one or more matched snippets (one per source kind).
Each snippet carries `score`, `start_sec` (for timestamp-linked transcript matches),
and optional `section_title`.

### `GET /api/search/status`

Returns `SearchStatusPayload`:

- `pending` / `indexing` / `ready` / `failed` counts
- `total_sources`, `total_chunk_count`, `embedded_chunk_count`
- `vector_index_ready`
- `retrieval_mode`
- `available` (whether any indexing has completed)

### `GET /api/search/status/stream`

Server-sent events stream of `SearchStatusPayload`. Emits on every status change from
the search index worker. Frontend uses this to update indexing progress live.

### `POST /api/search/rebuild`

Resets the entire derived search projection (clears `search_sources`) and re-initializes
progress tracking from canonical content. Used to force a full re-index after schema or
chunking changes.

---

## Status Surface

The frontend caches `search_status` independently during startup and refreshes it live
via the SSE stream. Indexing coverage updates are visible as soon as the worker completes
each batch, without requiring a page reload.

`SearchStatusPayload` fields:

| Field                  | Description                                                      |
| ---------------------- | ---------------------------------------------------------------- |
| `pending`              | Sources not yet indexed                                          |
| `indexing`             | Sources currently being claimed/processed                        |
| `ready`                | Sources successfully indexed                                     |
| `failed`               | Sources that hit an error on last attempt                        |
| `total_sources`        | Total unique `(video, source_kind)` pairs tracked                |
| `total_chunk_count`    | Total chunks written to S3 across all ready sources              |
| `embedded_chunk_count` | Chunks with an associated vector embedding                       |
| `vector_index_ready`   | Whether the ANN index exists and is queryable                    |
| `retrieval_mode`       | Current effective mode: `fts_only`, `hybrid_exact`, `hybrid_ann` |
| `available`            | True once any source has reached `ready`                         |
