# Workspace State Persistence

## Problem
Navigating away from the workspace page and returning resets context, forcing users to reselect channel, video, and mode.

## Goal
Preserve core workspace context across navigation so users return to the same working state.

## Requirements
- Persist selected channel id, selected video id, content mode, and shorts filter in browser storage.
- Restore persisted state on workspace page load before initial channel/video selection.
- Gracefully recover when persisted ids are stale or missing from current data.
- Keep existing loading/editing/formatting behavior intact.

## Non-Goals
- Multi-device sync.
- Persisting server-side user profiles.
- Reworking backend APIs.

## Design Considerations
Use localStorage in the page component to avoid backend coupling and preserve current architecture.

## Open Questions
- None.
