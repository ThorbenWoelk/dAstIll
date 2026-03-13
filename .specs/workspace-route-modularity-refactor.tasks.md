# Tasks: Workspace Route Modularity Refactor

## Current State

Workspace route decomposition is complete for this pass. The home route now delegates its header, channel sidebar, video rail, content pane, and mobile navigation to focused workspace components, and `frontend/src/routes/+page.svelte` is down to 1,792 lines from 3,278 after preserving orchestration logic in the route.

## Steps

- [x] Audit large frontend route files and identify refactor seams.
- [x] Extract pure workspace helpers into dedicated modules with targeted tests.
- [x] Extract the workspace header/search, channel sidebar, video rail, content pane, and mobile navigation into focused Svelte components.
- [x] Simplify `frontend/src/routes/+page.svelte` so it mainly coordinates data loading, persistence, and cross-panel actions.
- [x] Run formatting, type checks, unit tests, and production build verification.

## Decisions Made During Implementation

- This pass will focus on `frontend/src/routes/+page.svelte` as the primary refactor target.
- The queue route audit will be documented, but only low-risk shared extractions should touch it during this change.
- Route orchestration stays in the route unless moving logic into a pure helper or panel-local component clearly improves separation of concerns.
- Workspace presentation helpers are being extracted ahead of component moves so the structural refactor can reuse tested behavior instead of copying route-local functions.
- Panel-local UI state now lives with the panel components in Svelte 5 runes mode (`$props`, `$state`, `$derived`), while route persistence, selection, caching, and async content mutations remain in `frontend/src/routes/+page.svelte`.
- Shared workspace helpers now cover transcript presentation, summary quality presentation, channel filtering/sort cycling, and common date/duration formatting so those behaviors can be unit-tested outside the route.
- On mobile, the workspace header now matches the queue route’s top-row structure by using `ThemeToggle` plus `SectionNavigation`, with the search field moved into its own row below the shared top bar.
