# Tasks: DB Access Concurrency

## Current State

The scalability audit identified the backend's `tokio::Mutex<Connection>` as the next likely latency bottleneck after query indexing. Runtime DB access now uses clonable libsql connections instead of an async mutex, shared-handle coverage is in place, and backend verification passed including a release build.

## Steps

- [x] Add backend test coverage for shared multi-caller DB access.
- [x] Replace mutex-guarded DB state with clonable connection access.
- [x] Update handlers and workers to use the new DB access path.
- [x] Run backend format, tests, and release build verification.

## Decisions Made During Implementation

- Scope is limited to removing app-level DB serialization without changing API contracts.
- The runtime path now uses `DbPool::connect()` directly; a test-only `lock()` shim remains so existing DB unit tests can keep their current setup with minimal churn.
