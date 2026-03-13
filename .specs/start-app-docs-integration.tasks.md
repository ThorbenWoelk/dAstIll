# Tasks: Start App Docs Integration

## Current State
`start_app.sh` now starts backend, frontend, and docs together, and the local development docs match that workflow. Syntax verification passed, and both attached and detached startup smoke checks succeeded on alternate ports.

## Steps
- [x] Add docs service startup, cleanup, readiness checks, and logging to `start_app.sh`.
- [x] Update local development documentation to match the new startup flow.
- [x] Run syntax verification and a startup smoke check.

## Decisions Made During Implementation
- The docs process is started through the local `docs/node_modules/.bin/vitepress` binary so `DOCS_PORT` can be overridden without duplicating the hardcoded CLI flags from `docs/package.json`.
- `start_app.sh` now manages `docs.log` and includes docs in both the readiness summary and detached-mode supervision flow.
