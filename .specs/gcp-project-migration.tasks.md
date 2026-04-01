# Tasks: GCP Project Migration

## Current State
The migration is live. `dastill` now serves backend, frontend, and docs from real Artifact Registry images; Firestore in `dastill` now contains only `dastill_preferences`, `dastill_tts_stats`, and `dastill_videos`; and GitHub deploy secrets/vars point at `dastill`. The old shared project `uplifted-water-273221` was briefly scheduled for deletion by mistake on April 1, 2026, then immediately restored with billing re-linked; unrelated shared-project collections still live there, and only `dAstIll`-owned leftovers were removed. Repo-side `Validation` passed on `main`, and the remaining cutover issue is a `Release` workflow frontend env regression that blanked the backend/docs Cloud Run URLs until `deploy-frontend` was updated to resolve them locally.

## Steps
- [x] Create spec and task files for the GCP project migration.
- [x] Parameterize Terraform inputs for GitHub WIF binding and Firestore database location.
- [x] Align the deploy workflow around shared app/project naming locals instead of repeated literals.
- [x] Document the end-to-end migration and cutover steps for the new `dastill` project, including Firebase, Firestore, GitHub, Terraform state, and AWS caveats.
- [x] Validate the updated Terraform and workflow configuration locally.
- [x] Create the new `dastill` GCP project, attach billing, and enable the required APIs.
- [x] Apply Terraform in `dastill` to recreate service accounts, Cloud Run services, Firebase, WIF, secrets, and dependent IAM.
- [x] Export Firestore from `uplifted-water-273221`, restage it into an EU bucket, and import it into the new `dastill` Firestore database.
- [x] Remove non-`dastill_*` collection groups from `dastill` after the initial shared-project Firestore import brought over unrelated data.
- [x] Update GitHub Actions secrets and variables to target `dastill`.
- [x] Redeploy the docs service in `dastill` with a real container image.
- [x] Redeploy the backend and frontend services in `dastill` with real container images.
- [x] Verify the live app end-to-end against the new Firestore/Firebase project.
- [x] Restore `uplifted-water-273221` after the mistaken project-level delete request and stop project-wide teardown.
- [x] Audit whether any old `dastill`-specific resources still remain in `uplifted-water-273221` without touching unrelated shared-project workloads.
- [x] Remove confirmed `dastill` leftovers from the shared old project (`dAstIll Web`, `dastill-databricks-token`, and the `dastill_*` Firestore collections) without deleting the shared project itself.
- [x] Fix the `Release` workflow so `deploy-frontend` resolves backend/docs service URLs inside the same job instead of publishing them as job outputs that GitHub Actions may mask.
- [ ] Observe the `main` branch `Release` workflow for the frontend env-fix commit through completion so repo-side deployment catches up with the already-live cutover.

## Decisions Made During Implementation
- The new target project is `dastill`.
- The old `uplifted-water-273221` project must remain active because it hosts unrelated shared workloads; only `dAstIll`-owned data/resources should move or be removed.
- Firestore is now explicitly configurable and defaults to `eur3` for the new project example.
- The GitHub WIF IAM binding now defaults to the current project number and can be overridden for a separate pool-host project.
- The initial Firestore export/import was too broad because it came from a shared project; the new `dastill` database was corrected afterward by bulk-deleting every non-`dastill_*` collection group.
- The repo-side backend CI issue was fixed by removing the Firestore credential dependency from the chat ownership test, so `Validation` now passes on `main`.
- GitHub Actions treated the backend/docs Cloud Run URLs as secret-like when they were promoted to job outputs, so frontend deploys now resolve those URLs inside `deploy-frontend` and pass them directly into the Cloud Run deploy action.
