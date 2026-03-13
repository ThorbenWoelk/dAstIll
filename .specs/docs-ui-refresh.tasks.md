# Tasks: Docs Overview Page Refresh

## Current State
Scope narrowed to the overview page only. The VitePress home hero has been replaced with a documentation-first landing page, the overview cards include explicit dark-theme contrast styling, and the theme switch now avoids the clipped browser tooltip.

## Steps
- [x] Review the current docs implementation and define the refresh scope.
- [x] Rewrite the homepage into a documentation-first landing page.
- [x] Run formatting, build verification, and a local visual smoke check.

## Decisions Made During Implementation
- The docs shell will remain unchanged; only the overview page is being refreshed.
- The overview page now uses normal doc layout with page-scoped card styling instead of the VitePress home hero.
- The overview cards use explicit `.dark` overrides so dark mode does not inherit the light-surface treatment.
- The appearance switch mirrors its tooltip text into `aria-label` and removes the native `title` tooltip so the control does not render out of bounds near the viewport edge.
