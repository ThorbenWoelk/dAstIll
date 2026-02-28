# Tasks: Video Info Tab

## Current State
Implementation and verification complete. Backend and frontend checks pass locally.

## Steps
- [x] Create spec and tasks files.
- [x] Add backend video info response model and endpoint.
- [x] Implement YouTube watch-page metadata extraction for description/title/details.
- [x] Add frontend `info` tab mode with info panel rendering.
- [x] Run backend/frontend verification gates.

## Decisions Made During Implementation
- Video info is fetched on-demand instead of introducing a new metadata cache table.
- Backend falls back to stored DB video fields if live watch-page metadata fetch fails.
