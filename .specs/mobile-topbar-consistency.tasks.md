# Tasks: Mobile Top Bar Consistency

## Current State
Implementation and verification are complete. Mobile route navigation now collapses into a shared current-section dropdown in the top bar across workspace, queue, and highlights, while desktop keeps the pill nav. The route headers now use the same responsive structure, and the built-preview HTML confirms the section-navigation trigger renders on `/`, `/download-queue`, and `/highlights`.

## Steps
- [x] Add shared responsive top-bar classes for mobile route headers.
- [x] Apply the shared header structure to workspace, queue, and highlights while preserving route-specific controls.
- [x] Run frontend format, tests, checks, and build to verify the header changes.

## Decisions Made During Implementation
- Mobile route navigation uses a dropdown trigger showing the current section instead of a burger icon or a native select.
- Desktop keeps the existing tab-pill navigation, but it now reads from the same shared section-navigation source as mobile.
- The workspace search field stays in the header but moves onto a full-width wrapped row on mobile so it does not force horizontal overflow.
