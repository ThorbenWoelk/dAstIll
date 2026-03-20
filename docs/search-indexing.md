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

### Chunking Parameters

| Parameter                  | Value | Purpose                                             |
| -------------------------- | ----- | --------------------------------------------------- |
| `TRANSCRIPT_TARGET_WORDS`  | 300   | Target words per transcript chunk                   |
| `TRANSCRIPT_OVERLAP_WORDS` | 40    | Overlap words between consecutive transcript chunks |
| `SUMMARY_TARGET_WORDS`     | 300   | Target words per summary section chunk              |
| `EMBEDDING_DIMENSIONS`     | 512   | Vector dimensions for embeddinggemma                |
| `EMBED_BATCH_SIZE`         | 8     | Chunks per embedding API request                    |

### Transcript chunking

Transcript chunking is paragraph-aware and preserves context across chunk boundaries:

1. **Paragraph detection**: Splits input on blank lines to identify natural paragraph boundaries
2. **Paragraph grouping**: Accumulates paragraphs until adding another would exceed `TRANSCRIPT_TARGET_WORDS` (300)
3. **Long paragraph handling**: Paragraphs exceeding the target are split into word-based chunks
4. **Overlap injection**: After completing a chunk, the last `TRANSCRIPT_OVERLAP_WORDS` (40) words are carried forward to prefix the next chunk

This overlap ensures search queries that span chunk boundaries remain discoverable.

### Summary chunking

Summary chunking follows a two-tier approach:

1. **Full-document chunk**: Always creates one chunk containing the entire normalized summary text, marked with `is_full_document: true`. This chunk is never split and preserves the complete summary for broad queries.

2. **Section-based chunks**: Parses markdown `## ` headings to identify sections. For each section:
   - Sections under `SUMMARY_TARGET_WORDS` (300) become single chunks with the section title preserved
   - Sections over the target are split into word-based chunks (without overlap)
   - Each chunk carries its `section_title` for filtering and display

Section chunks are marked `is_full_document: false` and enable targeted queries within specific summary sections.

### Chunk Content Normalization

Before chunking, text is normalized:

- removes markdown heading prefixes (`#`, `##`, etc.)
- removes list markers (`-`, `*`, `1.`, `2)`)
- collapses whitespace and blank lines into single spaces
- strips leading/trailing whitespace

This produces clean searchable text without formatting artifacts.

## Retrieval Modes

The backend reports one of two retrieval modes:

| Mode         | Meaning                                             |
| ------------ | --------------------------------------------------- |
| `fts_only`   | Plain keyword search over chunk text                |
| `hybrid_ann` | ANN vector retrieval via S3 Vectors plus FTS fusion |

S3 Vectors provides native ANN search, replacing the previous exact vector rerank mode.

## Semantic Enablement Rules

The search service only generates embeddings when semantic search is enabled.

If semantic search is disabled:

- search sources are still chunked and indexed
- FTS still works
- `embedded_chunk_count` remains `0`

## Embedding Model

The default embedding model is **embeddinggemma:latest**, configured via `OLLAMA_EMBEDDING_MODEL`. The embedding service:

- calls Ollama's `/api/embed` endpoint
- batches embedding requests for efficiency (up to 8 chunks per request)
- validates embedding dimensions match the configured model (512 for embeddinggemma)

The embedding model must be pulled and available in Ollama before semantic search can function.

### Embedding Input Format

Before embedding, each chunk is enriched with metadata to improve semantic retrieval:

```text
Video: <video_title>
Channel: <channel_name>
Source: transcript|summary
Section: <section_title> (optional, for summary chunks)

<chunk_text>
```

This context prefix helps the embedding model produce vectors that capture not just the chunk content but its relationship to the video and channel.

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
