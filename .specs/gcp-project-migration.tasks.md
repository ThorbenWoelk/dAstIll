# Tasks: GCP Project Migration

## Current State
The migration is live. `dastill` now serves backend, frontend, and docs from real Artifact Registry images; Firestore data has been imported successfully into the new project; and GitHub deploy secrets/vars point at `dastill`. The old shared project `uplifted-water-273221` was briefly scheduled for deletion by mistake on April 1, 2026, then immediately restored with billing re-linked; only `dAstIll`-owned leftovers were removed there and unrelated shared-project workloads were left intact.

## Steps
- [x] Create spec and task files for the GCP project migration.
- [x] Parameterize Terraform inputs for GitHub WIF binding and Firestore database location.
- [x] Align the deploy workflow around shared app/project naming locals instead of repeated literals.
- [x] Document the end-to-end migration and cutover steps for the new `dastill` project, including Firebase, Firestore, GitHub, Terraform state, and AWS caveats.
- [x] Validate the updated Terraform and workflow configuration locally.
- [x] Create the new `dastill` GCP project, attach billing, and enable the required APIs.
- [x] Apply Terraform in `dastill` to recreate service accounts, Cloud Run services, Firebase, WIF, secrets, and dependent IAM.
- [x] Export Firestore from `uplifted-water-273221`, restage it into an EU bucket, and import it into the new `dastill` Firestore database.
- [x] Update GitHub Actions secrets and variables to target `dastill`.
- [x] Redeploy the docs service in `dastill` with a real container image.
- [x] Redeploy the backend and frontend services in `dastill` with real container images.
- [x] Verify the live app end-to-end against the new Firestore/Firebase project.
- [x] Restore `uplifted-water-273221` after the mistaken project-level delete request and stop project-wide teardown.
- [x] Audit whether any old `dastill`-specific resources still remain in `uplifted-water-273221` without touching unrelated shared-project workloads.
- [x] Remove confirmed `dastill` leftovers from the shared old project (`dAstIll Web`, `dastill-databricks-token`, and the `dastill_*` Firestore collections) without deleting the shared project itself.

## Decisions Made During Implementation
- The new target project is `dastill`.
- The old `uplifted-water-273221` project does not need to remain compatible after cutover.
- Firestore is now explicitly configurable and defaults to `eur3` for the new project example.
- The GitHub WIF IAM binding now defaults to the current project number and can be overridden for a separate pool-host project.
- The release workflow on `main` is currently not usable for cutover because one backend test tries to create a Firestore client without CI credentials; live deployment is proceeding via direct Cloud Build and `gcloud run deploy`.
