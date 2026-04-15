# Hot-Read Migration Boundary

This note defines which remaining `load_all_videos()` callers are acceptable to leave scan-backed for the current hot-read phase, and which ones still block the spec from being considered fully complete.

## First-wave request paths already moved off full scans

- Channel browse and paging use bounded per-channel `libSQL` / Turso windows.
- Recent library activity uses bounded per-channel `libSQL` / Turso windows.
- Chat video suggestions use a scoped cached catalog built from bounded per-channel windows plus direct `get_videos(...)` lookups for `Others`.
- Chat mention resolution now reuses that same scoped cached catalog instead of loading the full library on each request.
- Authenticated `Others` channel detection now looks up only the caller's saved membership IDs instead of scanning every video record.

## Remaining scan-backed callers that are acceptable for this phase

These paths are maintenance, startup, admin, or stats-oriented. They are not part of the first-wave user-facing hot-read cutover target.

- `backend/src/db/search.rs`
  - `list_search_backfill_materials`
  - `list_search_progress_materials`
  - `get_search_source_counts`
- `backend/src/workers/mod.rs`
  - FTS hydration fallback paths that bulk-load the library during worker startup
- `backend/src/db/video_info.rs`
  - `list_video_ids_missing_info`
  - `list_video_ids_for_info_refresh`
- `backend/src/db/stats.rs`
  - `videos_by_channel`
  - `count_resource_by_channel`
- `backend/src/db/videos/mod.rs`
  - `list_video_ids_by_channel` for the synthetic `__others__` scope during destructive cleanup flows

## Interactive follow-up work that still keeps the spec open

This is still an interactive read, so it is not covered by the maintenance/admin/stats boundary above.

- `backend/src/db/videos/mod.rs`
  - `has_unsubscribed_channel_videos` still scans the full catalog for the legacy virtual-`Others` detection path used by the older scoped-view helpers.

Until that legacy interactive caller is replaced or retired, the hot-read spec should be treated as in progress rather than fully implemented.
