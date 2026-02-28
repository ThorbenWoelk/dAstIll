# Tasks: UI Audit 2026

## Current State
Pending/loading visuals were refined to reduce UI clutter: status rows now emphasize non-ready states and loading is shown via skeleton placeholders. Verification passes: `bun run check` and `bun run build`.

## Steps
- [x] Review guideline rules and inspect current frontend files
- [x] Define and apply layout/alignment fixes in route-level UI
- [x] Apply shared component accessibility and interaction improvements
- [x] Polish global styles for motion, safe areas, and visual consistency
- [x] Improve pending/loading state visual clarity in cards and content panels
- [x] Run frontend verification (`check`, `build`) and summarize evidence

## Decisions Made During Implementation
- Keep existing visual direction (warm editorial look) and improve interaction quality instead of replacing the entire design language.
- Consolidate button/focus behavior in route-level class tokens to keep interaction affordances consistent.
- Move Google Fonts loading from CSS to `<head>` links to remove CSS import-order build warnings.
- Use skeleton placeholders and concise status text for pending/loading instead of badge-like controls to keep states obvious without looking interactive.
