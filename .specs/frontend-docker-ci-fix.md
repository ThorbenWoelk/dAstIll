# Frontend Docker CI Fix

## Problem

The frontend runtime Docker stage installs production dependencies with `npm install --omit=dev` even though the project uses `bun.lock` as the source of truth. That allows CI builds to resolve a different dependency graph from the one used during the build stage, which can fail on registry drift or changed transitive versions.

## Goal

Make the frontend Docker build use the locked Bun dependency graph for both build-time and runtime dependencies so CI image builds are deterministic.

## Requirements

- The runtime image must not run an unlocked `npm install`.
- Production dependencies for the runtime image must be derived from `bun.lock`.
- The frontend build output must remain compatible with the Node runtime image.

## Non-Goals

- No broader deployment workflow changes.
- No package dependency upgrades unrelated to the Docker install path.
