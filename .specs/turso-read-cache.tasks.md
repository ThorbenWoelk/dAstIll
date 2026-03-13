# Tasks: Turso Read Cache

## Current State
Implementation and verification complete. A 10-second in-memory backend cache now covers the hot workspace/channel reads plus search-status polling, with coarse invalidation on writes.

## Steps
- [x] Add a backend read-cache module with TTL and invalidation tests.
- [x] Mount the cache in backend app state and use it for the hot read endpoints.
- [x] Invalidate the cache on user-driven writes that affect channel/video list state.
- [x] Run backend and frontend verification gates.

## Decisions Made During Implementation
- Short-lived in-memory cache is preferred over a distributed cache for this pass because it is the smallest change with immediate read savings.
- `GET /api/search/status` is included because the workspace header polls it every 15 seconds and it otherwise adds steady Turso read pressure.
- Coarse cache invalidation remains acceptable because the read-heavy endpoints overlap strongly and the 10-second TTL bounds staleness even across worker-driven changes.
- Search status uses a 30-second TTL instead of the default 10 seconds so the workspace's 15-second poll cadence actually benefits from backend caching.
