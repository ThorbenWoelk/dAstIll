# Tasks: Release Workflow Fixes

## Current State

Workflow, Terraform, and deployment docs are patched. Targeted verification passed with `terraform fmt -check terraform/iam.tf`, scoped `git diff --check`, and `bunx prettier@3.6.2 --check` on the touched workflow/spec/docs files. Remaining rollout caveat: the Firebase Auth deploy fix depends on applying the Terraform IAM change before the next `Release` rerun.

## Steps

- [x] Inspect the failed `Release` workflow logs and identify the exact failing jobs.
- [x] Confirm the backend startup failure maps to the renamed Ollama env vars being blank in Cloud Run.
- [x] Patch the release workflow and Terraform IAM with the minimal compatible fix.
- [x] Run targeted verification for the touched workflow and Terraform files.
- [x] Update this task file with verification results and any remaining rollout caveats.
