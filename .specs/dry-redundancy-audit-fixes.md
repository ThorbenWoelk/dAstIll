# DRY And Redundancy Audit Fixes

**Linear:** none

## Problem

The app currently has duplicated workspace and content-update logic across frontend routes and backend handlers. That duplication has already caused behavior drift in the queue view, where refresh requests do not consistently preserve the active queue filter.

## Goal

Consolidate the duplicated logic identified in the audit so that shared behavior lives in shared helpers, queue refresh behavior stays consistent with the active filter, and manual content updates use a single backend path per content type.

## Requirements

- Queue refreshes on `/download-queue` must preserve the currently selected `queueTab` when fetching the post-refresh snapshot.
- Channel workspace state handling shared by the main workspace and queue page must be extracted into reusable helpers where practical, reducing route-local duplication for restore/persist, ordering, and refresh configuration.
- Manual transcript and summary updates must use shared backend persistence helpers instead of parallel hand-written flows.
- The content editor must render shared action controls from reusable definitions instead of maintaining duplicated markup for edit and read-only modes.
- Redundant frontend state identified in the audit must be removed when it has no effect on rendered behavior.
- Automated tests must cover the queue refresh filter regression and the extracted backend/content persistence behavior.

## Non-Goals

- No visual redesign outside the minimal UI changes needed to remove duplication safely.
- No API shape changes beyond what is required internally to share logic.
- No deployment or branch management changes.

## Design Considerations

- Prefer small reusable helpers over introducing a large cross-route store if the shared behavior can stay stateless.
- Preserve the current workspace state schema in localStorage unless a schema change is required for compatibility.
- Keep frontend refactors incremental so existing user-edited files are not rewritten wholesale.

## Open Questions

- None at the moment.
