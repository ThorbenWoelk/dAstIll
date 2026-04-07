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
| Content hash skip | Does the indexer skip chunks whose content hash hasn't changed, or does it always re-embed? |
| Chunk limits per file | Is there a hard cap on chunks per source? Or can a very long transcript produce hundreds of chunks? |
| Stale detection | When a video is deleted, are its chunks cleaned up from the search index? |
| Exclude lists | Are there any source types that get indexed but shouldn't? (e.g., failed transcripts, draft metadata) |

## Spec Items

### A1 — Verify Content Hash Gating

Check if the indexing worker compares content hashes before re-embedding. If not, add
`content_hash` to the chunk metadata and skip re-embedding when hash matches.

Files to check:
- `backend/src/services/search/mod.rs` (or wherever the indexing claim/process loop is)
- Any chunk or embedding persistence code

### A2 — Verify Chunk Count Caps

Check if there is a hard limit on chunks per source (transcript or summary). If a 3-hour
transcript produces 500 chunks, that is too many. Add a cap of 50–100 chunks per source,
with larger chunk sizes for very long content.

Files to check:
- Chunking logic in the search indexing pipeline
- `docs/search-indexing.md` for documented chunk strategy

### A3 — Verify Stale Cleanup

When a video or channel is unsubscribed/deleted, confirm that:
- FTS entries are removed
- Vector entries are removed
- S3 chunk JSON is cleaned up

Files to check:
- Delete handlers in the API
- Search index cleanup for orphaned entries

### A4 — Document Embedding Cost Profile

Add a section to `docs/search-indexing.md` documenting:
- Approximate embedding time per chunk batch
- Total embedding time for a typical full reindex
- Whether Ollama embeddings are local or server-side
- What the CPU/memory profile looks like during reindex

This helps future development decisions about when to offload or optimize.

## Priority

A1 and A2 are worth checking first. A3 is a correctness concern. A4 is documentation.

## Verification

- Run a full reindex and measure: time, CPU %, memory usage
- Introduce a duplicate transcript and confirm no re-embedding occurs (if A1 is implemented)
- Delete a video and confirm search index cleanup (A3)
