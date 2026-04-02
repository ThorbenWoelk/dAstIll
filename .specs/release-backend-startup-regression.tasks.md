# Tasks: Release Backend Startup Regression

## Current State
Validation passes locally, and the Release workflow is now patched to deploy backend and frontend runtime env vars via `env_vars_file` instead of inline `env_vars`. The failing Cloud Run revisions were traced to a malformed `RUST_LOG` value produced by the previous inline serialization path.

## Steps
- [x] Identify whether the failure is in Validation or Release.
- [x] Reproduce the workflow state locally and inspect recent GitHub Actions failures.
- [x] Confirm the backend revision fails before binding `PORT` and capture the Cloud Run revision logs.
- [x] Update the Release workflow to pass runtime env vars via env var files instead of inline `env_vars`.
- [x] Run targeted verification on the workflow change and inspect the resulting diff.

## Decisions Made During Implementation
- Treat this as a deployment-config regression, not a backend application-code regression.
- Prefer `env_vars_file` over value-specific escaping so future comma-containing env vars remain safe.
