# Local RAG Best Practices Audit — dAstIll

## Context

Reference: `totos-vault/Knowledge Base/AI/RAG/Local RAG Best Practices.md`

## Checklist Audit

### ✅ Already Implemented

| Practice | Status | Evidence |
|---|---|---|
| Background indexing | ✅ | Worker claims pending items, separate from serving |
| Batch embedding | ✅ | `SEARCH_EMBED_BATCH_SIZE` in embedding code |
| Hybrid retrieval | ✅ | BM25 + vector + RRF fusion |
| Graceful degradation | ✅ | System degrades to FTS-only if vector fails |
| Canonical/search separation | ✅ | S3 canonical, libSQL/Turso + S3 Vectors for search |
| Indexing status reporting | ✅ | Chat status payloads for retrieval planning |

### ⚠️ Should Verify

| Practice | Question |
|---|---|
| Content hash gating | Does the indexer avoid requeueing and re-embedding unchanged sources, and are sources only marked pending when content or embedding requirements actually changed? |
| Chunk limits per file | Is there a hard cap on chunks per source? Or can a very long transcript produce hundreds of chunks? |
| Stale detection | When a video is deleted, are its chunks cleaned up from the search index? |
| Exclude lists | Are there any source types that get indexed but shouldn't? (e.g., failed transcripts, draft metadata) |

## Spec Items

### A1 — Verify Source-Level Content Hash Gating

Confirm that the indexing worker uses source-level content hashes to avoid unnecessary
requeueing and re-embedding when a transcript or summary has not changed.

This item is about source-level gating, not chunk-level hash reuse. Do not add
per-chunk `content_hash` metadata or chunk-level embedding reuse unless profiling shows
that sources are being marked `pending` redundantly often enough to justify the added
complexity.

Specifically verify:
- unchanged ready sources are not requeued during reconcile
- changed content is requeued
- failed sources are requeued
- embedding model changes requeue sources when semantic search is enabled
- upstream write paths do not mark unchanged sources pending unnecessarily

Files to check:
- `backend/src/workers/search_index.rs`
- `backend/src/handlers/content/generation.rs`
- Search source state / persistence code

### A2 — Verify Chunk Count Caps

Check if there is a hard limit on chunks per source (transcript or summary). If a 3-hour
transcript produces 500 chunks, that is too many. Add a cap of 50–100 chunks per source,
with larger chunk sizes for very long content.

Files to check:
- Chunking logic in the search indexing pipeline
- `docs/search-indexing.md` for documented chunk strategy

### A3 — Verify Deletion Cleanup by Path

Split cleanup verification by deletion path instead of treating all stale-search cleanup
as one case.

#### A3.1 — Source Deletion (Transcript / Summary)

When a transcript or summary is deleted, confirm that the specific
`(video_id, source_kind)` cleanup removes:
- FTS entries
- Vector entries
- S3 chunk JSON / bundle objects
- Search source state

Files to check:
- `backend/src/db/content.rs`
- `backend/src/db/search.rs`
- `backend/src/services/fts.rs`

#### A3.2 — Channel / Video / Unsubscribe Cleanup

When a video is deleted or a channel is unsubscribed/deleted, confirm that cleanup
propagates across all affected videos and removes:
- FTS entries
- Vector entries
- S3 chunk JSON / bundle objects
- Search source state

Files to check:
- Delete / unsubscribe handlers in the API
- `backend/src/db/channels.rs`
- Shared search cleanup helpers

### A4 — Document the Benchmark Procedure, Not Fixed Cost Numbers

Add a short section to `docs/search-indexing.md` that explains how to measure indexing
and embedding cost in a given environment. Document:
- Which environment details materially affect results
  (`OLLAMA_URL`, `OLLAMA_EMBEDDING_MODEL`, hardware, semantic on/off, batch size)
- How to run and time a representative full reindex
- Which metrics to capture
  (wall-clock time, chunk count, embedding batch count, CPU, memory)
- Where to store a dated benchmark artifact or measurement note

Do not hard-code fixed timing, CPU, or memory numbers in the architecture doc unless
they are clearly labeled as dated sample measurements with environment details.

## Priority

A1 and A2 are worth checking first. A3 is a correctness concern. A4 is documentation.

## Verification

- Run a full reindex and capture a dated benchmark artifact with environment details
- Introduce a duplicate transcript and confirm no re-embedding occurs (if A1 is implemented)
- Delete a transcript or summary and confirm FTS + vector + S3 cleanup for that source
- Delete or unsubscribe a channel and confirm cleanup propagates to all affected videos
