# Data Model

## Canonical Tables

These tables represent application truth and are not merely cache artifacts.

| Table         | Role                                                                              |
| ------------- | --------------------------------------------------------------------------------- |
| `channels`    | Subscribed YouTube channels and sync depth                                        |
| `videos`      | Per-video state, publication metadata, queue status, acknowledgement, retry count |
| `transcripts` | Extracted raw text and formatted markdown transcript forms                        |
| `summaries`   | Generated or manually edited summaries plus quality fields                        |
| `video_info`  | Extended metadata such as description, duration, and view count                   |
| `highlights`  | User-created transcript or summary snippets with context                          |

## Core Status Fields

`videos` carries two key lifecycle fields:

- `transcript_status`
- `summary_status`

Each can be:

- `pending`
- `loading`
- `ready`
- `failed`

These statuses drive the queue worker and much of the UI state.

Additional video fields:

- `acknowledged` - tracks whether the user has marked a video as seen
- `retry_count` - caps regeneration attempts for summaries
- `quality_score` - 0-100 rating from the evaluator model

## Search Projection

Search is intentionally modeled as a derived projection stored in S3:

| Storage          | Role                                           |
| ---------------- | ---------------------------------------------- |
| `search_sources` | Per-video, per-source indexing lifecycle state |
| `search_chunks`  | Chunked search content stored as S3 objects    |
| S3 Vectors Index | Vector embeddings for semantic search          |

S3 Vectors provides managed ANN vector storage and retrieval, eliminating the need for a separate FTS5 table.

### `search_sources`

Tracks one record per `(video_id, source_kind)` pair with:

- `content_hash`
- `source_generation`
- `embedding_model` - stores which embedding model was used (default: embeddinggemma:latest)
- `index_status`
- `last_indexed_at`
- `last_error`

### `search_chunks`

Each chunk is stored as an S3 object with:

- `search_source_id`
- `source_generation`
- `chunk_index`
- `section_title`
- `chunk_text`
- `token_count`

Embeddings are stored separately in S3 Vectors.

## Highlights

The `highlights` table stores user-selected snippets:

- `id` - unique identifier
- `video_id` - associated video
- `source` - `transcript` or `summary`
- `text` - the highlighted content
- `prefix_context` / `suffix_context` - surrounding text for context
- `created_at` - timestamp

Highlights are grouped by channel and video in the `/highlights` route.

## Chat Storage

Chat conversations are stored in S3 as JSON objects, separate from the canonical tables:

| Storage                    | Role                                               |
| -------------------------- | -------------------------------------------------- |
| `conversations/index.json` | Conversation list index with titles and timestamps |
| `conversations/{id}.json`  | Full conversation with all messages and sources    |

### Conversation Structure

Each conversation contains:

- `id` - unique identifier
- `title` - auto-generated or user-set title
- `created_at` / `updated_at` - timestamps
- `messages` - ordered list of messages

### Message Structure

Each message includes:

- `id` - unique identifier
- `role` - `user` or `assistant`
- `content` - the message text
- `status` - `pending`, `streaming`, `complete`, or `failed`
- `sources` - retrieved chunks used for RAG grounding (assistant messages only)

Sources reference the search index and provide attribution for AI responses.

### Why Separate Chat Storage

Chat is intentionally separate from canonical content:

- conversations are ephemeral user interactions, not canonical content
- messages reference existing search chunks but don't duplicate them
- conversations can be deleted without affecting transcripts or summaries

## Why Separate Canonical and Search Tables

This lets the app:

- rebuild search without rewriting canonical content
- change chunking and indexing behavior independently
- keep user-facing writes fast
- isolate failures in embedding or search projection work

## Derived State Rules

### Canonical writes queue search work

Transcript and summary changes do not inline-rebuild embeddings. They mark the relevant search source pending.

### Search chunks are disposable

If the projection schema changes, the backend can drop and recreate `search_sources` and `search_chunks` while preserving canonical transcript and summary content. S3 Vectors embeddings can be rebuilt independently.

## Counting Search Coverage

Search coverage totals intentionally use readiness flags from `videos` rather than scanning large transcript or summary text tables. That keeps status payloads lightweight enough for startup surfaces.
