# Tasks: Hot-Read Video Views

## Current State
Most first-wave hot request paths have been moved off `load_all_videos()` full-collection reads, especially for channel browsing and chat suggestion/recent-activity flows. Remaining work is now concentrated in a small number of interactive follow-up callers plus documenting which maintenance/admin/stats scans are intentionally still acceptable.

## Steps
- [x] Replace the old Firestore-index scalability spec with a mixed index-plus-cache hot-read spec and task file.
- [x] Inventory request-path `load_all_videos()` consumers and rank them by user-facing impact.
- [x] Define the minimal Firestore index set that supports bounded per-channel ordered reads.
- [x] Shut off unused automatic single-field Firestore indexes in Terraform and explicitly allow only still-needed equality-query fields.
- [x] Replace the highest-priority request paths with bounded Firestore reads or bounded local cached reads while preserving route and payload contracts.
- [x] Add local cache-backed replacement for title-suggestion lookup rather than using Firestore full scans.
- [x] Replace chat mention-scope resolution and authenticated `Others` membership lookup with scoped catalog/direct-ID reads instead of full-library scans.
- [x] Replace interactive highlights grouping with direct highlighted-video and channel lookups instead of full-library scans.
- [x] Define how ingest, sync, and mutation paths invalidate or refresh those caches and local lookups without request-time full scans.
- [x] Define a refill or rebuild path from canonical Firestore when local lookup state is empty or stale.
- [x] Define the migration boundary for remaining offline, admin, or stats callers that can stay scan-backed for now.
- [ ] Finish or retire the remaining legacy virtual-`Others` detection scan so the spec can close cleanly.
- [x] Define verification for parity, stale-view recovery, and request-path performance improvement.

## Decisions Made During Implementation
- Firestore remains the source of truth.
- A small number of targeted Firestore indexes is allowed when it is cheaper than repeated full scans.
- Unused Firestore automatic indexes should be disabled through Terraform rather than left on by default.
- Local cache or local derived lookup state should handle title-suggestion style access patterns that Firestore still models poorly.
- The first pass targets the highest-impact user-facing request paths before lower-priority scan callers.
- Video suggestions now refill a per-scope in-process catalog cache on miss by walking channel-ordered Firestore windows and direct `get_videos(...)` lookups for `Others`.
- Channel browse and recent-activity reads now use bounded per-channel Firestore windows instead of loading the full video collection.
- Chat mention resolution now reuses the scoped suggestion catalog instead of loading the full video collection.
- Highlight grouping now resolves only the referenced videos and channels instead of reading the full library.
- The accepted scan-backed boundary is documented in `docs/architecture/hot-read-migration-boundary.md`; the only interactive scan still open is legacy virtual-`Others` detection.
