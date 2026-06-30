# Search

<script setup>
const queryPathDiagram = String.raw`
flowchart TB
  query[User query]
  keyword[Keyword search<br/>BM25 / FTS5]
  results[Search + chat results]

  query --> keyword
  keyword --> results
`;

const indexMaintenanceDiagram = String.raw`
flowchart TB
  canonical[Ready transcript or summary]
  pending[search_sources pending]
  worker[Search index worker]
  chunks[search_chunks in GCS]
  keyword[local libSQL FTS5]
  retrieval[Keyword retrieval]

  canonical --> pending
  pending --> worker
  worker --> chunks
  worker --> keyword
  keyword --> retrieval
`;
</script>

## Overview

Search currently runs as keyword search through local libSQL FTS5.

Semantic vector search is disabled in the GCS-only runtime. `SEARCH_SEMANTIC_ENABLED` must be
`false`, and `vector_index_ready` remains `false`.

<MermaidDiagram
  caption="Search query path: user queries run through the keyword index and return grouped video results."
  :chart="queryPathDiagram"
/>

## Storage And Rebuild Boundary

| Store           | Role                                                  |
| --------------- | ----------------------------------------------------- |
| GCS data bucket | Rebuild source for `search-chunks/` JSON objects      |
| local libSQL    | Runtime keyword index queried directly by the backend |

GCS remains the rebuild source of truth for chunk content. The local keyword index is rebuilt from
stored chunks when the runtime index is empty.

<MermaidDiagram
  caption="Index maintenance flow: canonical content becomes pending search sources, then the search worker chunks, stores, and syncs the libSQL FTS5 keyword index."
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
| Index pending sources | Claim pending rows, chunk content, write chunks                     |
| Reconcile             | Requeue stale rows after content hash or error changes              |
| Prune                 | Remove stale chunk rows no longer referenced by a ready generation  |

Summary sources are prioritized before transcript sources when discovering, claiming, and
reconciling work. This keeps summary searchability from waiting behind large transcript backlogs.

Canonical transcript and summary write paths mark sources pending. They do not chunk inline.

## Chunking

Chunk sizes and chunk caps live in [Runtime Limits](/operations/runtime-limits#search-limits).

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

## Query Path

<MermaidDiagram
  caption="Keyword retrieval searches the local FTS index and groups matching chunks by video."
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

### 2. Keyword Leg

BM25 search targets:

- `chunk_text`
- `video_title`
- `section_title`

Channel and source-kind filters are applied in SQL. Candidate limits live in
[Runtime Limits](/operations/runtime-limits#search-limits).

`extract_keyword_snippet` centers snippets around the earliest matching token. Long snippets are
trimmed to the configured snippet window.

The FTS pre-ranker sorts candidates before grouping:

1. Exact phrase match in chunk text, video title, or section title.
2. Summary source before transcript source.
3. All query terms present in the video title.
4. More query terms present in the video title.
5. Original BM25 rank.

### 3. Grouping

Results are grouped by `video_id`. Each group includes display metadata and up to one best match per
source kind. Response limits live in [Runtime Limits](/operations/runtime-limits#search-limits).

## Execution Modes

The effective runtime mode is `keyword`.

| Mode       | FTS | Semantic | Status                           |
| ---------- | --- | -------- | -------------------------------- |
| `keyword`  | Yes | No       | Supported                        |
| `semantic` | No  | No       | Disabled in the GCS-only runtime |
| `hybrid`   | Yes | No       | Degrades to keyword-only         |

## Status Surface

The runtime reports `fts_only` while semantic search is disabled.

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
