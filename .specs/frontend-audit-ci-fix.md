# Frontend Audit CI Fix

## Problem

The `Security / Frontend Audit` GitHub Actions job is failing on `main` because `bun audit --production` reports vulnerable `picomatch` transitive dependencies in the frontend lockfile.

## Goal

Refresh the frontend dependency resolution so the production audit passes again without changing unrelated application behavior.

## Requirements

- `frontend/bun.lock` must no longer resolve the vulnerable `picomatch@2.3.1` path reported by `bun audit --production`.
- `frontend` install, format, lint, type-check, tests, and production audit must pass locally after the change.
- The fix should stay scoped to frontend dependency resolution unless investigation proves a workflow change is required.

## Non-Goals

- No unrelated frontend refactors or feature work.
- No CI workflow redesign unless dependency-only remediation is not viable.
- No backend or docs dependency updates unless they are directly required for this failure.

## Design Considerations

The failure is currently caused by a vulnerable transitive dependency under `@vitejs/devtools` -> `unstorage` -> `anymatch` -> `picomatch`. The preferred fix is to refresh dependency resolution with the smallest lockfile and manifest change that removes the vulnerable version while keeping the toolchain stable.

## Open Questions

- Whether a lockfile refresh alone is enough, or whether an explicit `overrides` entry is required to force a patched `picomatch` for `anymatch`.
