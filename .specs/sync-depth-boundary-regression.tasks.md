# Tasks: Sync Depth Boundary Regression

## Current State
The main workspace now keeps the displayed sync boundary tied to the stored or derived channel boundary unless history was explicitly expanded by the user. Frontend tests, type-checking, and the production build all pass.

## Steps
- [x] Inspect the current sync-depth read/write paths and isolate the regression.
- [x] Add a failing regression test for sync-depth display behavior.
- [x] Implement the minimal frontend fix.
- [x] Run format, type-check, tests, and build verification.

## Decisions Made During Implementation
- Treat this as a display-boundary regression unless new evidence shows a backend write path outside explicit user actions.
- Gate loaded-video sync-depth overrides behind explicit history expansion in the main workspace and reset that allowance on passive snapshot or filter reloads.
