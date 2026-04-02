# GCP Project Migration

## Status

Cutover is operationally complete in `dastill`. Cloud Run, Firebase, secrets, local envs, and the app runtime now point at the new project; the `Release` workflow already picked up the frontend URL fix; the Firebase Google sign-in regression caused by an old-project OAuth client is fixed live and captured in repo config through `frontend/firebase.json` plus the release workflow's Firebase Auth deploy step; and Terraform now pins Firestore single-field exemption resources to `var.project_id` so those resources no longer drift back to `uplifted-water-273221`.

## Problem

The repo originally ran from the shared GCP project `uplifted-water-273221` and needed a clean cutover to a dedicated `dastill` project without leaving `dAstIll`-owned runtime state behind in the old shared environment. Most of that migration is complete; the remaining cleanup is Terraform/state ownership for Firestore field resources that still target the old project.

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
- Firebase Google sign-in is not just a generic OAuth client wiring problem. Reusing the old project's client breaks the Google redirect in the new project, and Terraform's generic IAM OAuth client resources do not create the same kind of Firebase web client that Google sign-in expects.
- Some Firestore resources, especially `google_firestore_field`, need `project = var.project_id` explicitly. Otherwise Terraform can keep targeting the old project from state even after the database resource itself has moved.

## Resolved Outcomes

- `dastill` exists, billing is attached, Firebase is enabled, and Terraform has recreated the required GCP resources there.
- The Firestore database in `dastill` uses the explicit configurable location path added by this migration work.
- GitHub WIF now resolves against configurable Terraform inputs instead of the old hard-coded project number.
- The shared old project remains active because it hosts unrelated workloads.
- `dastill` has been trimmed back to only `dastill_preferences`, `dastill_tts_stats`, and `dastill_videos` after the initial whole-export import brought along unrelated shared collections.
- Local backend and frontend env files now point at `dastill` instead of the old shared project, and the stale local Firestore service-account path was removed in favor of ADC.
- The old shared project no longer contains the `dastill_preferences`, `dastill_tts_stats`, or `dastill_videos` Firestore collection groups.
- The release workflow now resolves backend/docs service URLs inside the `deploy-frontend` job instead of publishing them as job outputs, preventing GitHub Actions from blanking the frontend `BACKEND_API_BASE`, `BACKEND_IDENTITY_AUDIENCE`, and `PUBLIC_DOCS_URL` env vars on deploy.
- Firebase Google sign-in in `dastill` now uses a valid project-local `apps.googleusercontent.com` client again. The source of truth is `frontend/firebase.json`, deployed with `firebase deploy --only auth`, instead of Terraform-managed OAuth client credentials copied from another project.
- Firestore single-field exemption resources now set `project = var.project_id` explicitly, so Terraform tracks those field overrides in `dastill` instead of inheriting stale state targeting from `uplifted-water-273221`.
