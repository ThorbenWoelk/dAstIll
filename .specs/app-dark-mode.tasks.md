# Tasks: App Dark Mode

## Current State

Docs-style dark mode is implemented in the product frontend. Theme preference is applied before hydration, a shared switch is present on the main app surfaces, and the shared theme switch now exposes only its accessible label without any visible tooltip.

## Steps

- [x] Audit the frontend for theme-sensitive colors and identify the shared dark-mode entry points.
- [x] Add tested theme preference helpers and early app-shell theme initialization.
- [x] Add a shared theme toggle and adapt the main app surfaces to semantic dark-mode tokens.
- [x] Run frontend format, tests, checks, build, and a local smoke verification.

## Decisions Made During Implementation

- Dark mode mirrors the docs behavior by storing only explicit `light` or `dark` preferences and otherwise following the system color scheme.
- Theme initialization happens in `src/app.html` so the `html.dark` class and `theme-color` meta are set before the Svelte app hydrates.
- The main workspace, queue, and highlights routes share the same theme switch component rather than duplicating theme logic per page.
- The app theme switch keeps its accessible `aria-label`, but does not render any visible tooltip because the control lives in the top header and the extra hover label adds noise.
