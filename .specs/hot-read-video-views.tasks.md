# Tasks: Hot-Read Video Views

## Current State
Audit complete. Hot request paths still rely on `load_all_videos()` full-collection reads, especially in channel browsing and chat-related flows. New Firestore indexes are off the table, so the replacement path will keep Firestore canonical and use Turso or libsql-backed materialized read views instead of indexed Firestore queries.

## Steps
- [x] Replace the old Firestore-index scalability spec with a no-index hot-read-views spec and task file.
- [ ] Inventory request-path `load_all_videos()` consumers and rank them by user-facing impact.
- [ ] Define the Turso or libsql-backed hot-read catalog schema for the first-pass priority flows.
- [ ] Define the minimal hot-read view schemas needed for channel browse snapshots or paging, chat suggestions or mention resolution, and recent-library-activity reads.
- [ ] Decide whether channel browse also needs pre-shaped per-channel snapshot records on top of the Turso catalog for the most latency-sensitive reads.
- [ ] Define how ingest, sync, and mutation paths update those materialized views without introducing request-time full scans.
- [ ] Define the rebuild or backfill path from canonical Firestore and existing derived data when hot-read views are missing or stale.
- [ ] Replace the highest-priority request paths with hot-read view reads while preserving route and payload contracts.
- [ ] Define the migration boundary for remaining offline, admin, or stats callers that can stay scan-backed for now.
- [ ] Define verification for parity, stale-view recovery, and request-path performance improvement.

## Decisions Made During Implementation
- No new Firestore indexes will be introduced for this problem.
- Firestore remains the source of truth; Turso or libsql hot-read views are derived state.
- This work does not migrate all Firestore data into Turso.
- The first pass targets the highest-impact user-facing request paths before lower-priority scan callers.
