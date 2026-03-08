# Tasks: Loading Performance Overhaul

## Current State
Combined bootstrap/snapshot endpoints are in place, the frontend now uses them for initial paint, and automated verification plus a backend smoke test passed locally.

## Steps
- [x] Add regression tests for the faster startup/bootstrap flow and any new backend query behavior.
- [x] Implement a combined backend bootstrap path for channels, AI availability, selected channel sync depth, and initial videos.
- [x] Remove redundant frontend startup and channel-selection fetches in favor of the bootstrap path and cached refresh behavior.
- [x] Improve backend database access and indexes for channel/video read paths.
- [x] Run formatting, type checks, tests, and production builds for frontend and backend.

## Decisions Made During Implementation
- Focus first on app-controlled latency rather than external summarizer or YouTube refresh latency.
- Replace `(? IS NULL OR column = ?)` video filters with dynamic SQL so the new composite indexes are usable.
- Keep YouTube refresh in the background after first paint instead of blocking initial content visibility on it.
