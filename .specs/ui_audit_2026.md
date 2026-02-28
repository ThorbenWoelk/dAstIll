## Goal

Upgrade the Svelte frontend so it feels production-grade for 2026: cleaner alignment, stronger visual hierarchy, faster scanability, and better accessibility.

## Scope

- Audit current UI against modern web interface guidelines.
- Fix alignment and spacing inconsistencies in layout, cards, and action rows.
- Improve UX details: focus states, input semantics, status feedback, empty states, and loading behavior.
- Keep the current product structure and data flows intact.

## Non-Goals

- No backend API changes.
- No new product features beyond UX polish and interaction quality.
- No visual redesign that breaks the existing brand direction.

## Acceptance Criteria

- Main layout is consistently aligned on mobile and desktop.
- Interactive controls have visible focus and clear disabled/hover states.
- Forms and controls include required accessibility semantics (`name`, `autocomplete`, labels, ARIA where needed).
- Media elements avoid layout shift (image dimensions/loading behavior).
- Error and loading feedback are more usable and screen-reader friendly.
- Frontend passes type check and production build.
