# GCP Project Migration

## Status

Cutover is operationally complete in `dastill`. Cloud Run, Firebase, secrets, and the app runtime all point at the new project. The remaining repo-side work is keeping the `Release` workflow healthy so frontend redeploys continue to inject the backend/docs Cloud Run URLs correctly after GitHub masked the earlier cross-job URL outputs.

## Problem

The application is still configured and operated from the old GCP project `uplifted-water-273221` ("Totos Home"). The repo already parameterizes many resources by `project_id`, but deployment, IAM, and migration documentation still contain project-specific assumptions that block a clean cutover to a new GCP project named `dastill`.

## Goal

Make the repo ready to run from the `dastill` GCP project, with Terraform, GitHub Actions, Firebase/Firestore wiring, and migration documentation aligned so the app can be cut over to the new project while leaving unrelated workloads in the shared old project alone.

## Requirements

- Terraform must no longer hard-code the old GitHub Workload Identity Federation pool project number or repository binding.
- Firestore placement must be configurable so the new project can choose the intended database location explicitly.
- Deployment workflow configuration must be centralized enough that project/app identifiers and secret mount names are not scattered across the workflow logic.
- Operations documentation must describe the required cutover steps for the new GCP project, including Terraform state handling, GitHub secrets/vars updates, Firebase setup, and Firestore data migration.
- The migration documentation must call out the AWS ownership caveat: the repo also manages shared AWS resources whose names are derived from `app_name`, so migration must either reuse/import that state or intentionally re-home those resources.

## Non-Goals

- Automatically creating the new GCP project, attaching billing, or enabling Firebase terms acceptance.
- Fully automating Firestore cross-project export/import or GitHub secret rotation from the repo.
- Deleting the shared `uplifted-water-273221` project or modifying unrelated workloads/data that still live there.

## Design Considerations

- Most runtime code is already env-driven (`GCP_PROJECT_ID`, `PUBLIC_FIREBASE_PROJECT_ID`, backend/frontend URLs), so the main repo work is infra parameterization and migration clarity rather than app logic rewrites.
- The GitHub WIF principal should default to the target GCP project when possible, but remain overrideable so the identity pool can live in a separate host project if needed.
- Firestore location should be explicit because database location is effectively permanent once created.
- AWS resources are global/account-scoped and currently keyed by `app_name`, not `project_id`. That makes them a migration dependency even though the user only asked for a GCP project move.
- The old Firestore export came from a shared database. That means cutover needs either collection-scoped migration or a post-import cleanup pass so `dastill` only keeps `dAstIll` collections.

## Resolved Outcomes

- `dastill` exists, billing is attached, Firebase is enabled, and Terraform has recreated the required GCP resources there.
- The Firestore database in `dastill` uses the explicit configurable location path added by this migration work.
- GitHub WIF now resolves against configurable Terraform inputs instead of the old hard-coded project number.
- The shared old project remains active because it hosts unrelated workloads.
- `dastill` has been trimmed back to only `dastill_preferences`, `dastill_tts_stats`, and `dastill_videos` after the initial whole-export import brought along unrelated shared collections.
- The release workflow now resolves backend/docs service URLs inside the `deploy-frontend` job instead of publishing them as job outputs, preventing GitHub Actions from blanking the frontend `BACKEND_API_BASE`, `BACKEND_IDENTITY_AUDIENCE`, and `PUBLIC_DOCS_URL` env vars on deploy.
