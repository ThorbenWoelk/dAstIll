# Hot-Read Video Views

## Problem

Several request-path backend flows still rely on loading the entire Firestore video collection and then filtering and sorting in memory. That creates latency and scalability risk in hot user-facing paths such as channel browsing, chat suggestions, recent-activity chat flows, and other scoped video lookups.

The earlier all-no-index direction was too strict. At the current library size, a small number of targeted Firestore indexes is cheaper than continuing to burn read quota on full-collection scans. The real problem is broad read fanout on hot paths, not raw Firestore storage volume.

## Goal

Replace request-path full-collection video scans in the highest-traffic user-facing flows with a mixed strategy:

- a minimal set of Firestore indexes for bounded per-channel ordered reads
- aggressive local caching for hot repeated reads
- local derived metadata for cases where Firestore indexing is still a poor fit

This must preserve existing endpoint behavior while keeping Firestore as the canonical source of truth.

## Requirements

- Replace request-path `load_all_videos()` usage in the highest-priority user-facing flows with bounded reads.
- Keep Firestore as the canonical source of truth for video records.
- Introduce only a small number of Firestore indexes whose read savings clearly outweigh their storage and write costs.
- Shut off unused Firestore automatic single-field indexes via Terraform rather than keeping broad defaults.
- Use Firestore for bounded per-channel ordered reads where it is a good fit.
- Use local cached metadata or local derived lookup state where Firestore indexing is still a poor fit, especially title suggestion paths.
- Ensure request-path reads use direct-key lookups, bounded Firestore queries, or bounded local cached queries rather than full collection scans.
- Preserve existing HTTP routes, request parameters, and response payload shapes for the migrated flows.
- Define how caches and local derived lookups are invalidated or refreshed when video metadata changes, new videos are ingested, or user-visible status fields change.
- Define a rebuild or refill path for any local derived lookup state when it is empty or stale.
- Define the migration boundary for remaining offline, admin, stats, or low-priority callers that may remain scan-backed for now.
- Define verification criteria for correctness parity, stale-read handling, and request-path performance improvement.

## Non-Goals

- Redesigning the full storage architecture or moving away from Firestore as the source of truth.
- Migrating all Firestore data or all application state into Turso.
- Reintroducing broad automatic Firestore indexing across the whole `dastill_videos` schema.
- Reworking every remaining batch, admin, or stats code path in the same pass.
- Changing UI behavior or introducing new API paging semantics.
- Generalizing the whole library model beyond current video and channel flows in this pass.

## Design Considerations

- Firestore should only answer query shapes that map cleanly to a very small index set. In this pass that mainly means per-channel reads ordered by `published_at`.
- Single-field indexes should be explicitly allowlisted for fields still used by existing Firestore equality queries, rather than relying on broad automatic defaults.
- The repo already has local cache and libsql or Turso runtime wiring. Those should be used for title-suggestion and repeated hot-read paths before introducing more Firestore indexes.
- Local derived lookup state should stay intentionally narrow: only the fields needed for hot request paths belong in it.
- Update paths should be tied to existing ingest, sync, and mutation flows so the system does not depend on request-time repair.
- The first pass should prioritize user-facing latency and read-cost wins over architectural completeness.

## Open Questions

- Should title-suggestion metadata stay in in-process cache only, or should it be promoted to a durable local libsql or Turso table if scope sizes or cold-start rates grow?
