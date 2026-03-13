# Workspace Route Modularity Refactor

**Linear:** none

## Problem

The main workspace route at `frontend/src/routes/+page.svelte` has grown into a 3,278-line component that mixes route hydration, API orchestration, optimistic updates, content formatting state, search behavior, panel layout, and multiple large conditional views. The size and coupling make the route hard to reason about, hard to safely change, and difficult to test. The audit also found `frontend/src/routes/download-queue/+page.svelte` at 1,297 lines, which indicates repeated shell patterns and route-local UI logic are starting to accumulate elsewhere as well.

## Goal

Restructure the workspace route so it remains the orchestration boundary but no longer acts as a single god component. Route-local UI concerns should live in focused Svelte components, reusable pure logic should live in helper modules with tests, and the resulting structure should make future changes to channel browsing, video browsing, and content viewing easier to implement without re-reading a monolith.

## Requirements

- The home workspace keeps its existing user-facing behavior for channel management, video browsing, content modes, search, highlights, editing, and guide navigation.
- `frontend/src/routes/+page.svelte` is reduced substantially by extracting focused components for major UI regions instead of keeping all markup inline.
- Pure logic currently embedded in the route script is moved into reusable helper modules where that improves clarity and testability.
- Newly extracted pure helpers are covered by targeted unit tests.
- The refactor preserves existing workspace URL state, local workspace persistence, and async loading behavior.
- The queue route audit is captured so follow-up work can address it deliberately, but this refactor pass does not need to fully decompose the queue route unless a small shared extraction clearly reduces duplication without broadening risk.
- Frontend verification covers formatting, type checking, unit tests, and production build.

## Non-Goals

- Reworking the visual design or changing the information architecture of the workspace.
- Replacing the existing API layer or changing backend behavior.
- Fully rewriting `frontend/src/routes/download-queue/+page.svelte` in the same pass.
- Introducing a global workspace store unless it is required to preserve behavior cleanly.

## Design Considerations

- Keep route orchestration close to the route: selection, loading, persistence, and mutation flows can stay in `+page.svelte` if they are coordinating multiple panels.
- Move UI state closer to the component that owns it when that state is purely presentational or panel-local, such as search popover open state or local filtering controls.
- Prefer extracting pure formatting and presentation helpers first so tests can lock behavior before more structural moves.
- Use component boundaries that align with the existing three-column workspace layout: header, channel sidebar, video rail, content pane, and mobile navigation.
- Avoid creating abstractions shared with the queue route unless the duplication is already clear and the shared component remains simple.

## Open Questions

- None at the moment. The audit gives a clear first-pass decomposition without needing product decisions from the user.
