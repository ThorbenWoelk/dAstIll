# Tasks: Hot-Read Video Views

## Current State
Audit complete. Hot request paths still rely on `load_all_videos()` full-collection reads, especially in channel browsing and chat-related flows. The updated direction is to use a very small set of targeted Firestore indexes where they clearly beat scan costs, shut off unused automatic indexes, and rely on local caching or local derived lookup state for the rest.

## Steps
- [x] Replace the old Firestore-index scalability spec with a mixed index-plus-cache hot-read spec and task file.
- [x] Inventory request-path `load_all_videos()` consumers and rank them by user-facing impact.
- [x] Define the minimal Firestore index set that supports bounded per-channel ordered reads.
- [x] Shut off unused automatic single-field Firestore indexes in Terraform and explicitly allow only still-needed equality-query fields.
- [x] Replace the highest-priority request paths with bounded Firestore reads or bounded local cached reads while preserving route and payload contracts.
- [x] Add local cache-backed replacement for title-suggestion lookup rather than using Firestore full scans.
- [x] Define how ingest, sync, and mutation paths invalidate or refresh those caches and local lookups without request-time full scans.
- [x] Define a refill or rebuild path from canonical Firestore when local lookup state is empty or stale.
- [ ] Define the migration boundary for remaining offline, admin, or stats callers that can stay scan-backed for now.
- [x] Define verification for parity, stale-view recovery, and request-path performance improvement.

## Decisions Made During Implementation
- Firestore remains the source of truth.
- A small number of targeted Firestore indexes is allowed when it is cheaper than repeated full scans.
- Unused Firestore automatic indexes should be disabled through Terraform rather than left on by default.
- Local cache or local derived lookup state should handle title-suggestion style access patterns that Firestore still models poorly.
- The first pass targets the highest-impact user-facing request paths before lower-priority scan callers.
- Video suggestions now refill a per-scope in-process catalog cache on miss by walking channel-ordered Firestore windows and direct `get_videos(...)` lookups for `Others`.
- Channel browse and recent-activity reads now use bounded per-channel Firestore windows instead of loading the full video collection.
