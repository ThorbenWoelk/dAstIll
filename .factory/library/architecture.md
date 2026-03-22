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

- Workspace route now uses SSR bootstrap loading via `src/routes/+page.server.ts` (main page no longer purely client-side)
- Current SSR bootstrap request forwards `selected_channel_id` + `limit`; URL `type`/`ack` filters are not currently forwarded in server load
- `api.ts`: 30-second in-memory GET response cache with request deduplication and targeted invalidation helpers for channel/video/highlight mutations
- `clearGetRequestCache()` remains as a test utility path (`resetApiCacheForTests`), not the primary mutation invalidation strategy
- `workspace-cache.ts`: IndexedDB persistence for stale-while-revalidate warm-start fallback and returning visits
- `getWorkspaceBootstrap` endpoint is used for SSR initial load and client-side bootstrap refresh
- Fraunces + Manrope are self-hosted WOFF2 assets; Google Fonts/Inter/Newsreader CDN references removed

## Frontend Component Structure

- Main page (+page.svelte): ~2200 lines, 50+ state variables, handles all workspace logic inline
- Routes: / (workspace), /highlights, /download-queue, /chat, /channels/[id], /login, /logout
- Service worker: no-op passthrough (sw.js)
- AI status polling: duplicated across routes (shared poller + highlights' own setInterval)
