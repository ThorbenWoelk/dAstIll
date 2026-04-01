# Tasks: UI Tour Refresh

## Current State
The docs-side UI tour has been rewritten around fresh April 1, 2026 captures from the deployed app, and the updated page builds successfully. Remaining work is limited to final verification notes and cleanup.

## Steps
- [x] Create spec and task files for the UI tour refresh.
- [x] Capture fresh desktop and mobile screenshots from the current app.
- [x] Rewrite the docs UI tour page from scratch around the current product surfaces.
- [x] Add or adjust docs theme styles needed for the new tour layout.
- [x] Align in-app guide copy only if the docs rewrite exposes material mismatches.
- [x] Run relevant docs/frontend verification and record the results.

## Decisions Made During Implementation
- The primary deliverable is the docs-side `UI Tour` page, because that surface already exists in the product documentation and explicitly references screenshots.
- Fresh screenshots should come from the current running app during this task rather than reusing older checked-in assets.
- The screenshots for this refresh were captured from the deployed Cloud Run frontend because local backend startup in this session is blocked by missing credentials.
- The in-app guide did not require copy changes for this pass because the rewritten docs stayed aligned with its existing feature claims.
- `docs` builds cleanly, and the touched docs files pass targeted Prettier checks. The repo-wide `docs` format check still fails because of unrelated pre-existing files outside this change.
