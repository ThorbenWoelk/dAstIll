# DB Access Concurrency

**Linear:** none

## Problem

Backend handlers and workers currently funnel all database work through a single `tokio::Mutex<Connection>`. Even when queries are individually small and paginated, that mutex serializes unrelated requests and background jobs, which can inflate latency as traffic and queue activity grow.

## Goal

Remove application-level serialization around database access so independent requests can reach libsql concurrently, while preserving current query behavior and testability.

## Requirements

- Backend state must stop using an async mutex around the shared DB handle.
- Callers must be able to obtain a reusable DB connection handle without waiting on unrelated operations.
- Automated coverage must prove the shared DB handle can be cloned and used across multiple callers against the same dataset.
- Existing handlers, workers, and tests must keep their behavior.

## Non-Goals

- No API shape changes.
- No query rewrite or pagination redesign in this change.
- No switch to a different database library.

## Design Considerations

- Prefer a small structural change that removes app-level contention without forcing a full storage abstraction rewrite.
- Keep the in-test in-memory DB behavior intact.

## Open Questions

- None.
