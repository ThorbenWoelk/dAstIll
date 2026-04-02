# Tasks: Environment Config Hygiene

## Current State
Cleanup complete. Local ignored `.env` files, repo-managed deploy config, GitHub Actions variable names, and live Cloud Run services have been aligned. Secret Manager contained no orphaned app secrets beyond the active set, so no secret deletions were required.

## Steps
- [x] Inventory env-var usage from application code, local `.env` files, Terraform, and GitHub Actions deploy config.
- [x] Inventory current Cloud Run env vars and secret bindings for backend, frontend, and docs services.
- [x] Inventory relevant Secret Manager secrets and map each one to active runtime consumers or stale status.
- [x] Remove or update stale repo-managed env configuration and documentation.
- [x] Remove safe stale remote Cloud Run env settings and Secret Manager secrets.
- [x] Verify the resulting local and remote config surfaces are aligned and document any intentionally retained exceptions.

## Decisions Made During Implementation
- Secret values must not be read or printed during the audit.
- GitHub Actions Cloud Run deployments should use authoritative env var replacement so blank or removed keys do not persist between releases.
- `PUBLIC_FIREBASE_API_KEY` is the only supported frontend Firebase web API key env name going forward.
- No Secret Manager deletions were needed because the live project only contained active app secrets referenced by Terraform and Cloud Run.
