# Tasks: Three Pane Transcript Workspace

## Current State
Implemented 3-pane layout with transcript availability filters in the videos pane.
Frontend verification passes with `bun run check` and `bun run build`.

## Steps
- [x] Define spec and requirements in `.specs/three-pane-transcript-layout.md`
- [x] Refactor page layout to left channel pane, middle transcript pane, right videos pane
- [x] Add transcript availability filter controls (all, available, unavailable) in videos pane
- [x] Ensure selected video state remains valid when filters/channels change
- [x] Run frontend verification (`bun run check`, `bun run build`)

## Decisions Made During Implementation
- Default transcript filter is `available` to prioritize transcript-ready videos.
