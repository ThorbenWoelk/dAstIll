# Mobile Top Bar Consistency

**Linear:** none

## Problem

The route-level top bar does not collapse consistently on mobile across the workspace, queue, and highlights sections. Each page uses slightly different wrapping and control placement, so the header shifts structure when moving between tabs.

## Goal

On mobile, the top bar should keep the same layout pattern across the main app sections while preserving each page's specific controls.

## Requirements

- The workspace, queue, and highlights routes all use the same mobile header structure for brand/status and route navigation controls.
- Route navigation pills remain readable and accessible on narrow screens without awkward wrapping.
- Page-specific controls, such as the workspace search input, still render without breaking the shared mobile layout.
- Desktop layout and existing behavior remain intact.
- Frontend format, tests, type checks, and production build pass after the change.

## Non-Goals

- Redesigning the visual style of the header.
- Changing route destinations or navigation labels.
- Adding new guide flows or new mobile actions.

## Design Considerations

Keep the fix lightweight by introducing shared responsive header classes rather than reworking the entire page shell. The shared mobile structure should allow a full-width secondary row when a route needs extra controls, such as workspace search.

## Open Questions

- None.
