# Firestore Query Scalability

## Problem

Several request-path backend flows still rely on loading the entire Firestore video collection and then filtering and sorting in memory. That approach may have been chosen to avoid index costs, but it now creates scalability and latency risk in hot user-facing paths such as channel browsing, chat suggestions, recent-activity chat flows, and channel deletion.

## Goal

Replace request-path full-collection video scans with indexed Firestore query paths for the highest-traffic user-facing flows while preserving current endpoint behavior.

## Requirements

- Add the required Firestore indexes in Terraform for hot query paths.
- Add paged and filtered Firestore query helpers for user-facing channel and video reads.
- Replace request-path `load_all_videos()` usage in the highest-priority handlers and chat services.
- Preserve existing endpoint paths and response payload shapes.
- Define the boundary between request-path migrations in this pass and remaining batch/admin/stats callers left for later.

## Non-Goals

- Redesigning the full storage architecture or moving away from Firestore.
- Reworking every remaining batch or admin code path in the same pass.
- Changing UI behavior or introducing new paging semantics at the API level.

## Design Considerations

- The repo already manages Firestore field index exemptions through Terraform, so index additions should be managed in the same place.
- The first pass should prioritize hot request paths, not background jobs.
- Any helper introduced here should be shaped around existing endpoint semantics so the frontend contract remains stable.

## Open Questions

- None at the moment. The immediate target paths and migration boundary are clear enough for a focused scalability pass.
