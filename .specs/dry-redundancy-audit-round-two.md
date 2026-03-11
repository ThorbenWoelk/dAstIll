# DRY And Redundancy Audit Round Two

**Linear:** none

## Problem

The remaining DRY issues are concentrated in shared channel workspace behavior across the two frontend routes and in parallel request/persistence helpers on the backend. The app works, but the same behaviors still exist in multiple places, which keeps configuration, filtering, and lookup logic easy to drift apart.

## Goal

Consolidate the remaining duplicated channel workspace, query parsing, lookup, and content persistence logic so that each behavior has one shared implementation and the existing route/API behavior remains unchanged.

## Requirements

- The main workspace and download queue routes must use shared helpers for persisted channel workspace state restoration, channel drag/drop state transitions, and channel refresh TTL decisions.
- Backend channel bootstrap/snapshot endpoints and the channel video list endpoint must share one query/filter contract for pagination, video type filters, queue filters, and acknowledged filtering.
- Backend channel and video existence lookups must use shared helpers instead of repeating the same DB fetch + 404 handling across handlers.
- Backend transcript and summary content write flows must expose one clear public write path per behavior, removing redundant DB helper entrypoints that only differ in status side effects.
- Existing behavior around queue-tab preservation, manual transcript render modes, and manual summary quality reset must remain covered by automated tests.

## Non-Goals

- No user-facing redesign.
- No API shape changes for clients beyond internal refactors.
- No deployment, branch, or infrastructure changes.

## Design Considerations

- Prefer pure helpers and small shared structs over introducing new stores or framework-heavy abstractions.
- Keep localStorage schema compatibility intact.
- Reduce public backend helper surface area where duplicate functions only exist for tests or legacy internal call sites.

## Open Questions

- None at the moment.
