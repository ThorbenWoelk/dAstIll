# Hot-Read Video Views

## Problem

Several request-path backend flows still rely on loading the entire Firestore video collection and then filtering and sorting in memory. That creates latency and scalability risk in hot user-facing paths such as channel browsing, chat suggestions, recent-activity chat flows, and other scoped video lookups.

The original fix direction was to add Firestore indexes for those paths. That is no longer acceptable because Firestore indexes are too expensive for this repo's operating constraints.

## Goal

Replace request-path full-collection video scans in the highest-traffic user-facing flows with app-managed materialized hot-read views backed primarily by Turso or libsql, without requiring new Firestore indexes, while preserving existing endpoint behavior and keeping Firestore as the source of truth.

## Requirements

- Do not add new Firestore indexes as part of this work.
- Replace request-path `load_all_videos()` usage in the highest-priority user-facing flows with reads against bounded, materialized hot-read views.
- Keep Firestore as the canonical source of truth for video records.
- Use Turso or libsql as the primary queryable backing store for derived hot-read views in this pass.
- Define one or more hot-read view shapes that support at least:
  - channel browse snapshots or paging
  - chat suggestions and mention resolution
  - recent-library-activity queries
- Ensure request-path reads use direct-key lookups, bounded reads, or bounded local/Turso queries rather than full collection scans.
- Preserve existing HTTP routes, request parameters, and response payload shapes for the migrated flows.
- Define how hot-read views are updated when video metadata changes, new videos are ingested, or user-visible status fields change.
- Define a rebuild or backfill path so the hot-read views can be recreated from existing canonical data if they are empty, stale, or corrupted.
- Define the migration boundary for remaining offline, admin, stats, or low-priority callers that may remain scan-backed for now.
- Define verification criteria for correctness parity, stale-read handling, and request-path performance improvement.

## Non-Goals

- Redesigning the full storage architecture or moving away from Firestore as the source of truth.
- Migrating all Firestore data or all application state into Turso.
- Adding new Firestore composite or single-field indexes to support the hot paths.
- Reworking every remaining batch, admin, or stats code path in the same pass.
- Changing UI behavior or introducing new API paging semantics.
- Generalizing the whole library model beyond current video and channel flows in this pass.

## Design Considerations

- The repo already has Turso or libsql runtime wiring for search, so a small metadata-oriented hot-read catalog in Turso is a viable option for bounded query paths without introducing another major dependency.
- Turso should be treated as a derived read-model store, not the new system of record. Firestore remains authoritative and Turso remains rebuildable.
- Direct per-channel materialized snapshots may still be useful as a local shaping detail, but the first-pass queryable backing store should be Turso or libsql rather than new Firestore query paths.
- The read model should stay intentionally narrow: only fields needed for hot request paths belong in the hot-read views.
- Firestore remains authoritative. Materialized views are derived state and must be rebuildable.
- Update paths should be tied to existing ingest, sync, and mutation flows so the system does not depend on request-time repair.
- The first pass should prioritize user-facing latency wins over architectural completeness.

## Open Questions

- Should channel browse use only Turso-backed queryable metadata, or should it also maintain pre-shaped per-channel snapshot records for the most latency-sensitive views?
