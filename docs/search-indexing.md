# Search Indexing

## Storage Backend

Search uses AWS S3 for chunk storage and AWS S3 Vectors for semantic embeddings:

- **S3 Data Bucket**: stores chunked search content as JSON objects
- **S3 Vectors Bucket/Index**: stores and indexes embeddings for ANN retrieval

This eliminates the need for a local FTS5 table and enables managed vector search.

## Search Worker Phases

The search worker is a loop with four recurring responsibilities.

### Backfill

Discovers canonical transcript and summary content that should already be searchable but has no `search_sources` row yet.

### Index Pending Sources

Claims pending rows, loads canonical content, chunks it, optionally embeds it, and writes the derived projection.

### Reconcile

Finds stale indexed rows and requeues them when:

- content hashes changed
- indexing previously failed
- the stored embedding model no longer matches the runtime model

### Prune

Removes stale chunk rows that are no longer referenced by a ready source generation.

## Summary Priority

Indexing intentionally prioritizes summaries before transcripts when:

- discovering missing work
- claiming pending work
- selecting reconciliation work

This keeps summary searchability from being starved behind a large transcript backlog.

## Chunking Strategy

### Transcript chunking

- paragraph-aware where possible
- approximately 300 target words
- approximately 40-word overlap

### Summary chunking

- always includes one full-document chunk
- also creates section-based chunks from markdown headings
- section chunks are split further only when necessary

## Retrieval Modes

The backend reports one of two retrieval modes:

| Mode           | Meaning                                             |
| -------------- | --------------------------------------------------- |
| `fts_only`     | Plain keyword search over chunk text                |
| `hybrid_ann`   | ANN vector retrieval via S3 Vectors plus FTS fusion |

S3 Vectors provides native ANN search, replacing the previous exact vector rerank mode.

## Semantic Enablement Rules

The search service only generates embeddings when semantic search is enabled.

If semantic search is disabled:

- search sources are still chunked and indexed
- FTS still works
- `embedded_chunk_count` remains `0`

## Query Path

At query time the backend:

```text
1. normalizes the query into an FTS expression
2. loads FTS candidates
3. optionally embeds the query
4. runs exact or ANN vector retrieval
5. fuses rankings
6. groups matches by video
```

## Status Surface

`/api/search/status` reports:

- `pending`
- `indexing`
- `ready`
- `failed`
- `total_sources`
- `total_chunk_count`
- `embedded_chunk_count`
- `vector_index_ready`
- `retrieval_mode`

The workspace bootstrap also includes `search_status` so indexing progress can appear immediately on first render.
