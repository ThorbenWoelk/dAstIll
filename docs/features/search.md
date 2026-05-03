# Search

<script setup>
const queryPathDiagram = String.raw`
flowchart TB
  query[User query]
  keyword[Keyword leg<br/>BM25 / FTS5]
  semanticprep[Optional HyDE<br/>+ embedding]
  vectors[Vector leg]
  fusion[RRF fusion]
  rerank[Optional reranker]
  results[Search + chat results]

  query --> keyword
  query --> semanticprep
  semanticprep --> vectors
  keyword --> fusion
  vectors --> fusion
  fusion --> rerank
  rerank --> results
`;

const indexMaintenanceDiagram = String.raw`
flowchart TB
  canonical[Ready transcript or summary]
  pending[search_sources pending]
  worker[Search index worker]
  chunks[search_chunks]
  keyword[libSQL FTS5]
  vectors[S3 Vectors]
  retrieval[Keyword + hybrid retrieval]

  canonical --> pending
  pending --> worker
  worker --> chunks
  worker --> keyword
  worker --> vectors
  keyword --> retrieval
  vectors --> retrieval
`;
</script>

## Overview

Search has three layers:

1. **BM25 keyword search** through local libSQL FTS5.
2. **Vector search** through S3 Vectors.
3. **RRF fusion**, optionally followed by a cross-encoder reranker.

Each layer degrades independently. If semantic search is unavailable, keyword search still works.

<MermaidDiagram
  caption="Search query path: keyword and semantic legs run independently, then merge before results are returned to workspace search or chat."
  :chart="queryPathDiagram"
/>

## Storage And Rebuild Boundary

| Store          | Role                                                  |
| -------------- | ----------------------------------------------------- |
| S3 data bucket | Rebuild source for `search-chunks/` JSON objects      |
| local libSQL   | Runtime keyword index queried directly by the backend |
| S3 Vectors     | Dense embeddings for ANN retrieval, keyed by chunk ID |

S3 remains the rebuild source of truth for chunk content. The local keyword index is rebuilt from
stored chunks when the runtime index is empty.

<MermaidDiagram
  caption="Index maintenance flow: canonical content becomes pending search sources, then the search worker chunks, embeds, stores, and syncs the libSQL FTS5 keyword index."
  :chart="indexMaintenanceDiagram"
/>

## Keyword Index

The keyword index stores searchable text and display metadata in one FTS5 document so a query can
return snippets without a second metadata lookup.

| Field           | Tokenizer   | Role                                                    |
| --------------- | ----------- | ------------------------------------------------------- |
| `chunk_id`      | stored only | Unique chunk identifier                                 |
| `video_id`      | stored only | Parent video                                            |
| `channel_id`    | stored only | Parent channel for post-filtering                       |
| `source_kind`   | stored only | `transcript` or `summary`                               |
| `source_key`    | `raw`       | Composite `{video_id}_{source_kind}` for exact deletion |
| `chunk_text`    | `en_stem`   | Primary BM25 text                                       |
| `section_title` | `en_stem`   | Summary section heading                                 |
| `video_title`   | `en_stem`   | Video title                                             |
| `channel_name`  | stored only | Display channel name                                    |
| `published_at`  | stored only | Sort/display publication date                           |
| `start_sec`     | stored only | Transcript timestamp                                    |

### Startup Hydration

`populate_fts_index_from_store` runs only when the runtime keyword index is empty.

1. List `search-chunks/` objects.
2. Fetch chunks concurrently.
3. Group chunks by `(video_id, source_kind)`.
4. Load video and channel metadata for each group.
5. Call `fts.upsert_source`.

When indexed rows already exist locally, startup uses the local index immediately.

### Live Sync

The search worker keeps the keyword index synchronized after every projection write:

- `fts.upsert_source` replaces rows for a `(video_id, source_kind)` pair after chunk writes.
- `fts.delete_source` removes rows when content is removed or empty.

Upsert always deletes old rows for the source key before inserting the new set.

## Search Worker

The search worker is a background loop with four recurring phases.

| Phase                 | Work                                                                |
| --------------------- | ------------------------------------------------------------------- |
| Backfill              | Find ready transcript/summary content without `search_sources` rows |
| Index pending sources | Claim pending rows, chunk content, embed when enabled, write chunks |
| Reconcile             | Requeue stale rows after content hash, error, or embedding changes  |
| Prune                 | Remove stale chunk rows no longer referenced by a ready generation  |

Summary sources are prioritized before transcript sources when discovering, claiming, and
reconciling work. This keeps summary searchability from waiting behind large transcript backlogs.

Canonical transcript and summary write paths mark sources pending. They do not chunk or embed inline.

## Chunking

Chunk sizes, chunk caps, embedding dimensions, and embedding batch size live in
[Runtime Limits](/operations/runtime-limits#search-limits).

### Transcript Chunks

Timed chunking is preferred when caption segments with timestamps are available from the `yt-dlp`
`json3` fallback path. Chunks inherit `start_sec` from the first segment they contain.

Paragraph chunking is used for plain text:

1. Split on blank lines.
2. Accumulate paragraphs to the target size.
3. Split oversized paragraphs by words.
4. Carry the configured overlap into the next chunk.

For very long transcripts, the worker raises the effective target size so the final projection stays
within the transcript chunk cap.

### Summary Chunks

Summary chunking writes:

- one full-document chunk with `is_full_document: true`
- section chunks split on `## ` headings

The full-document chunk is always retained. Section chunks preserve `section_title` and are capped by
the summary chunk cap.

### Text Normalization

`normalize_source_text` removes searchable formatting noise before chunking:

- markdown heading prefixes
- list markers
- excess blank lines and whitespace
- leading/trailing whitespace

## Embedding Input

Chunks are enriched with metadata before embedding:

```text
Video: <video_title>
Channel: <channel_name>
Source: transcript|summary
Section: <section_title>  (omitted when empty)

<chunk_text>
```

This moves vectors toward the source topic and improves recall for queries that reference the content
area instead of exact transcript wording.

The embedding service:

- reads the configured embedding model
- calls Ollama `/api/embed`
- batches chunks before each request
- validates returned dimensions
- checks model availability through Ollama `/api/tags`

Embedding batch and dimension limits live in [Runtime Limits](/operations/runtime-limits#search-limits).

## Query Path

<MermaidDiagram
  caption="Keyword and semantic retrieval run as separate legs. Hybrid mode fuses them and can rerank the fused candidate list."
  :chart="queryPathDiagram"
/>

### 1. Tokenization

The raw query is tokenized with a stopword-aware tokenizer. Meaningful terms are deduplicated and
capped for FTS matching and snippet centering. The term cap lives in
[Runtime Limits](/operations/runtime-limits#search-limits).

```text
"rust rust tokio axum libsql semantic search" -> ["rust", "tokio", "axum", "libsql"]
"what is the best db in town"                 -> ["db", "town"]
```

### 2. HyDE

HyDE runs when:

- a HyDE model is configured
- semantic search is enabled for the request
- the query stays within the HyDE term gate

The backend calls Ollama `/api/generate` to create a short hypothetical answer passage. That passage
becomes the embedding input. The original query still drives FTS and snippet extraction. The timeout
and term gate live in [Runtime Limits](/operations/runtime-limits#search-limits).

HyDE failure falls back to embedding the raw query.

### 3. Keyword Leg

BM25 search targets:

- `chunk_text`
- `video_title`
- `section_title`

Channel and source-kind filters are applied in SQL. Candidate limits depend on execution mode and
live in [Runtime Limits](/operations/runtime-limits#search-limits).

`extract_keyword_snippet` centers snippets around the earliest matching token. Long snippets are
trimmed to the configured snippet window.

The FTS pre-ranker sorts candidates before fusion:

1. Exact phrase match in chunk text, video title, or section title.
2. Summary source before transcript source.
3. All query terms present in the video title.
4. More query terms present in the video title.
5. Original BM25 rank.

### 4. Semantic Leg

The semantic leg requires semantic search enabled and an embedding model configured.

| Retrieval mode | Mechanism                                         |
| -------------- | ------------------------------------------------- |
| `hybrid_ann`   | ANN query via S3 Vectors                          |
| `hybrid_exact` | Exact dot-product scan via S3 before ANN is ready |

Both paths accept metadata filters for `source_kind` and `channel_id`.

Special case: `source=all` with `hybrid_exact` scans summaries only to keep latency bounded. The ANN
path handles all source kinds.

### 5. Fusion

Reciprocal Rank Fusion merges FTS and semantic lists:

```text
score(chunk) = sum over each list L where chunk appears:
               1 / (60.0 + rank_in_L)
```

Chunks found by both retrievers accumulate both contributions.

### 6. Reranking

The neural reranker runs when:

- a reranker model is configured
- execution mode is `hybrid`
- both FTS and semantic candidate lists are non-empty

The backend posts the capped fused chunks and the original query to Ollama `/api/rerank`. Results
sort by `relevance_score`. Rerank candidate and timeout limits live in
[Runtime Limits](/operations/runtime-limits#search-limits).

Reranker failure falls back to plain RRF ordering.

### 7. Grouping

Results are grouped by `video_id`. Each group includes display metadata and up to one best match per
source kind. Response limits live in [Runtime Limits](/operations/runtime-limits#search-limits).

## Execution Modes

| Mode       | FTS | Semantic | Fusion | Reranker      |
| ---------- | --- | -------- | ------ | ------------- |
| `keyword`  | Yes | No       | No     | No            |
| `semantic` | No  | Yes      | No     | No            |
| `hybrid`   | Yes | Yes      | RRF    | If configured |

If semantic search is unconfigured or embedding fails, `hybrid` degrades to FTS-only for that
request. If either candidate list is empty, the other list is used directly.

## Semantic Enablement

The search service only generates embeddings when semantic search is enabled.

If semantic search is disabled:

- sources are still chunked and indexed in libSQL FTS5
- FTS still works
- `embedded_chunk_count` remains `0`
- `vector_index_ready` remains `false`

Backend config can override the default. Local debug runs default semantic on. Release builds default
semantic off.

## Status Surface

The runtime reports the effective retrieval mode:

| Status mode    | Condition                                                  |
| -------------- | ---------------------------------------------------------- |
| `fts_only`     | Semantic search disabled, or no embedding model configured |
| `hybrid_exact` | Semantic enabled; ANN index not ready                      |
| `hybrid_ann`   | Semantic enabled; ANN index ready                          |

The reranker does not change `retrieval_mode`.

`SearchStatusPayload` also reports indexing counts:

- `pending`
- `indexing`
- `ready`
- `failed`
- `total_sources`
- `total_chunk_count`
- `embedded_chunk_count`
- `vector_index_ready`
- `available`

The frontend receives search status in workspace bootstrap and refreshes it through the status SSE
stream.

## API Entry Points

The backend exposes:

- `GET /api/search`
- `GET /api/search/status`
- `GET /api/search/status/stream`
- `POST /api/search/rebuild`

The live OpenAPI document is the debugging source of truth for parameter and response shape:

```text
/api/openapi.json
```

`POST /api/search/rebuild` resets the derived search projection and re-initializes progress tracking
from canonical content.
