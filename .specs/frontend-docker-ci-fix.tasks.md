# Tasks: Frontend Docker CI Fix

## Current State

Replaced the frontend runtime install path with a Bun lockfile-based production dependency stage and verified the production install path locally without lifecycle scripts.

## Steps

- [x] Replace the runtime `npm install` path with Bun lockfile-based production dependencies.
- [x] Verify the frontend build and the production Bun install path locally.

## Decisions Made During Implementation

- The runtime image now copies `node_modules` from a dedicated `prod-deps` stage built from `bun.lock`, instead of performing a fresh `npm install`.
- The production dependency stage uses `--ignore-scripts` because runtime dependencies do not need the app-level `prepare` hook.
