# GCP Project Migration

## Problem

The application is still configured and operated from the old GCP project `uplifted-water-273221` ("Totos Home"). The repo already parameterizes many resources by `project_id`, but deployment, IAM, and migration documentation still contain project-specific assumptions that block a clean cutover to a new GCP project named `dastill`.

## Goal

Make the repo ready to run from the `dastill` GCP project, with Terraform, GitHub Actions, Firebase/Firestore wiring, and migration documentation aligned so the app can be cut over to the new project without preserving the old environment.

## Requirements

- Terraform must no longer hard-code the old GitHub Workload Identity Federation pool project number or repository binding.
- Firestore placement must be configurable so the new project can choose the intended database location explicitly.
- Deployment workflow configuration must be centralized enough that project/app identifiers and secret mount names are not scattered across the workflow logic.
- Operations documentation must describe the required cutover steps for the new GCP project, including Terraform state handling, GitHub secrets/vars updates, Firebase setup, and Firestore data migration.
- The migration documentation must call out the AWS ownership caveat: the repo also manages shared AWS resources whose names are derived from `app_name`, so migration must either reuse/import that state or intentionally re-home those resources.

## Non-Goals

- Automatically creating the new GCP project, attaching billing, or enabling Firebase terms acceptance.
- Fully automating Firestore cross-project export/import or GitHub secret rotation from the repo.
- Preserving the old `uplifted-water-273221` environment after the cutover.

## Design Considerations

- Most runtime code is already env-driven (`GCP_PROJECT_ID`, `PUBLIC_FIREBASE_PROJECT_ID`, backend/frontend URLs), so the main repo work is infra parameterization and migration clarity rather than app logic rewrites.
- The GitHub WIF principal should default to the target GCP project when possible, but remain overrideable so the identity pool can live in a separate host project if needed.
- Firestore location should be explicit because database location is effectively permanent once created.
- AWS resources are global/account-scoped and currently keyed by `app_name`, not `project_id`. That makes them a migration dependency even though the user only asked for a GCP project move.

## Open Questions

- Whether the `dastill` GCP project already exists with billing enabled and Firebase activated.
- Whether the new Firestore database should use `eur3` or another location.
- Whether the GitHub WIF pool/provider will live inside the `dastill` project or a separate host project.
