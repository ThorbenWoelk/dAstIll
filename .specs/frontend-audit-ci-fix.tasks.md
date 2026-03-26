# Tasks: Frontend Audit CI Fix

## Current State
Frontend audit fixed by removing optional Vite devtools, regenerating `frontend/bun.lock`, and validating format, lint, type-check, tests, audit, and pre-commit locally.

## Steps
- [x] Inspect the latest CI failure and reproduce `bun audit --production` locally.
- [x] Trace the vulnerable dependency chain in the frontend lockfile.
- [x] Update frontend dependency resolution to eliminate the vulnerable `picomatch` path.
- [x] Run frontend format, lint, check, tests, and production audit locally.
- [x] Stage changed files and run `bash scripts/githooks/pre-commit`.

## Decisions Made During Implementation
- Keep the scope to frontend dependency resolution unless a lockfile-only fix proves impossible.
- Removed optional `@vitejs/devtools` instead of forcing a global `picomatch` override, because Bun does not support nested `overrides` and the vulnerable chain came only from the devtools dependency tree.
- Updated `resolveGuideStepFromUrl` to accept either a raw query string or a `URL` instance so the existing tests stay stable across Bun URLSearchParams behavior.
