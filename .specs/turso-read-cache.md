# Turso Read Cache

## Problem

The app is hitting managed Turso read limits during normal UI usage. The frontend already caches GET requests per browser session for 30 seconds, but the backend still executes the same hot read queries for every client, refresh, and cold navigation.

## Goal

Reduce repeated Turso reads on the hottest UI endpoints by introducing a short-lived backend read cache with explicit invalidation on user-driven writes.

## Scope

- Add an in-memory TTL cache inside the Rust backend process.
- Cache the main workspace and queue read endpoints that fan out into multiple DB reads:
  - `GET /api/channels`
  - `GET /api/workspace/bootstrap`
  - `GET /api/channels/{id}/snapshot`
  - `GET /api/channels/{id}/sync-depth`
  - `GET /api/search/status`
- Invalidate the cache on user-driven writes that change channel/video list state.

## Non-Goals

- Cross-instance distributed caching.
- Cache persistence across deploys or restarts.
- Caching every GET endpoint in the API.
- Replacing the existing frontend cache.

## Design

- Use a shared `ReadCache` in `AppState`, backed by `tokio::sync::RwLock<HashMap<...>>`.
- Use a short TTL so background-worker updates converge quickly even without direct invalidation.
- Use coarse invalidation (`clear`) after writes instead of maintaining fine-grained dependency graphs.
- Keep cached values typed and cloneable rather than serializing payloads.

## Risks

- Cached workspace/bootstrap payloads also include AI and search status, so those fields may be stale for up to the TTL.
- In multi-instance deployments, each instance keeps its own local cache, so hit rate depends on request distribution.

## Verification

- Unit tests for cache TTL and invalidation behavior.
- Backend tests, release build, frontend checks/tests/build, and docs build.
