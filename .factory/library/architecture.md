# Architecture

Architectural decisions, patterns discovered, and design constraints.

**What belongs here:** Architecture patterns, component boundaries, data flow, caching strategies.

---

## System Overview

- SvelteKit frontend (TypeScript, Svelte 5, Tailwind v4)
- Rust backend (Axum, Tokio, AWS S3/S3 Vectors)
- Frontend proxies all API calls through /api/[...path] server route to backend
- No traditional database - S3 objects as primary storage, S3 Vectors for semantic search

## Backend Data Layer

- `db/helpers.rs`: `load_all` uses bounded concurrency (`JoinSet` + `Semaphore`, default max concurrent S3 ops: 12)
- `db/videos.rs`: `bulk_insert_videos` uses bounded concurrent S3 put operations
- Channel snapshot loading reuses a single loaded video slice per channel (no duplicate `load_all_videos` call for oldest-ready + list)
- `get_video` supports skipping summary S3 reads when summary content is not requested
- `read_cache.rs` uses targeted eviction methods (`evict_channel`, `evict_channel_list`, `evict_video_content`, `evict_highlights`, `evict_search`, `evict_chat`) instead of global clear-on-mutation
- Workspace bootstrap cache key is request-param based (`selected_channel_id` may be `null`), while payload selection can resolve `null` to the first channel; invalidation logic must account for this distinction
- `cache_headers.rs` middleware applies `Cache-Control` directives by method/path family for GET routes

## Frontend Data Layer

- All route data loaded client-side in `onMount()` - zero SSR data loading
- `api.ts`: 30-second in-memory GET response cache with request deduplication
- `clearGetRequestCache()` wipes entire cache on every mutation (13 call sites)
- `workspace-cache.ts`: IndexedDB persistence for stale-while-revalidate warm-start
- `getWorkspaceBootstrap` endpoint exists but is unused on main page
- Google Fonts loaded via render-blocking stylesheet links (Fraunces+Manrope in layout, stale Inter+Newsreader in app.html)

## Frontend Component Structure

- Main page (+page.svelte): ~2200 lines, 50+ state variables, handles all workspace logic inline
- Routes: / (workspace), /highlights, /download-queue, /chat, /channels/[id], /login, /logout
- Service worker: no-op passthrough (sw.js)
- AI status polling: duplicated across routes (shared poller + highlights' own setInterval)
