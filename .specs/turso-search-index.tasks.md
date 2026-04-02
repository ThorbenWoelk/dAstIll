# Tasks: Turso Search Index

## Current State
Turso/libSQL now backs the keyword index through the existing `FtsIndex` boundary, startup bootstraps from stored search projection only when the runtime index is empty, and the production/runtime wiring is now in place across Terraform, Secret Manager, and the release workflow. Additional backend hardening landed for FTS mutation failure propagation, reset ordering, and S3 key parsing with underscored video IDs. Full repo verification and the final commit/push remain.

## Steps
- [x] Remove the in-progress S3 snapshot persistence changes that no longer apply.
- [x] Add Turso/libSQL runtime configuration and initialize the keyword index through the existing `FtsIndex` service.
- [x] Implement full-text storage/query operations against the Turso-backed index and keep rebuild hydration as a bootstrap path.
- [x] Update docs to describe the Turso search architecture and required runtime configuration.
- [x] Run targeted backend validation for the new search path.
- [x] Wire Turso production config through Terraform, Secret Manager, and the release workflow.
- [ ] Run the full repository verification gate, then commit and push the completed Turso integration.

## Decisions Made During Implementation
- Use Turso embedded replicas for the runtime search index so reads stay local while writes persist remotely.
- Keep S3 `search_chunks` / `search-bundles` as the rebuild source of truth in case the runtime keyword index starts empty.
