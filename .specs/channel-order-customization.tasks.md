# Tasks: Custom Channel Ordering in Left Sidebar

## Current State
Implementation complete. Channel list supports drag-and-drop reorder with persisted local workspace order.

## Steps
- [x] Create spec and tasks files.
- [x] Implement drag-and-drop UI hooks in channel card and page.
- [x] Persist channel order in workspace localStorage snapshot and reconcile on load.
- [x] Verify frontend check/build.

## Decisions Made During Implementation
- Persist ordering locally in workspace state instead of adding backend ordering APIs.
- Reorder logic reconciles saved IDs with fetched channels, dropping stale IDs and appending newly discovered channels.
