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
| `highlights`  | User-created transcript or summary snippets                                       |

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

## Search Projection Tables

Search is intentionally modeled as a derived projection.

| Table               | Role                                             |
| ------------------- | ------------------------------------------------ |
| `search_sources`    | Per-video, per-source indexing lifecycle state   |
| `search_chunks`     | Chunked search content plus optional embeddings  |
| `search_chunks_fts` | External-content FTS5 table over `search_chunks` |

### `search_sources`

Tracks one row per `(video_id, source_kind)` pair with:

- `content_hash`
- `source_generation`
- `embedding_model`
- `index_status`
- `last_indexed_at`
- `last_error`

### `search_chunks`

Stores:

- `search_source_id`
- `source_generation`
- `chunk_index`
- `section_title`
- `chunk_text`
- `token_count`
- `embedding`

### `search_chunks_fts`

FTS5 is maintained as an external-content projection of `search_chunks`, with insert/update/delete triggers.

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

If the projection schema changes, the backend can drop and recreate `search_sources`, `search_chunks`, and `search_chunks_fts` while preserving canonical transcript and summary content.

## Counting Search Coverage

Search coverage totals intentionally use readiness flags from `videos` rather than scanning large transcript or summary text tables. That keeps status payloads lightweight enough for startup surfaces.
