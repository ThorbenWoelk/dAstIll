# Video List Query Scalability

**Linear:** none

## Problem

The workspace and queue views correctly paginate video rows, but the backend still relies on a small set of broad indexes. As the `videos` table grows, filtered list queries can degrade because the planner has no composite index for some common UI filter combinations.

## Goal

Keep workspace and queue list queries stable as video history grows by adding indexes that match the current filter patterns and verifying they exist locally.

## Requirements

- Backend migrations must create composite indexes for the filtered channel video list queries the frontend already issues.
- Automated coverage must verify the new indexes are present after DB initialization.
- Existing query behavior and API shapes must remain unchanged.

## Non-Goals

- No pagination API redesign in this change.
- No frontend UI changes.
- No DB engine migration or pooling redesign in this change.

## Design Considerations

- Prefer low-risk schema/index changes that improve current query shapes without widening scope.
- Focus on channel-scoped list queries first because they drive the initial workspace and queue loads.

## Open Questions

- None.
