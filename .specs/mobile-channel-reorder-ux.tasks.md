# Tasks: Mobile Channel Reorder UX

## Current State

Implementation and verification are complete. The workspace sidebar now exposes a dedicated mobile reorder mode with touch drag handles, visible drop markers, move up/down fallback controls, and a lifted drag preview, and frontend tests/checks/build all passed.

## Steps

- [x] Add failing frontend tests for the new reorder helper behavior and mode gating.
- [x] Implement a touch-friendly mobile reorder mode with visible handle, active drag state, and drop target feedback.
- [x] Add a non-drag fallback for mobile reordering while preserving custom-order persistence.
- [x] Run frontend format, tests, checks, and build verification.

## Decisions Made During Implementation

- Mobile reorder UX will use an explicit reorder mode instead of piggybacking on default card taps.
- The new reorder callbacks resolve a full next channel order in the sidebar and hand that order back to the route, so drag and non-drag reorder paths share the same persistence flow.
- Mobile drag feedback uses a dedicated handle, an insertion line based on the current reorder direction, and a floating “Moving” chip instead of attempting full live list virtualization during touch drag.
