# Tasks: Firebase Auth Release Decoupling

## Current State

Workflow, Terraform, docs, and spec tracking are updated. Normal releases no longer deploy Firebase Auth or wait on a Firebase Auth job, and targeted validation passed with `terraform fmt -check`, `bunx prettier@3.6.2 --check`, and scoped `git diff --check` on the touched files.

## Steps

- [x] Remove Firebase Auth deployment from the normal `Release` workflow path.
- [x] Update deployment docs so Firebase Auth config is described as a separate setup or maintenance step.
- [x] Run targeted validation on the touched workflow and docs files.
- [x] Update this task file with the implementation result and verification evidence.

## Decisions Made During Implementation

- Normal app releases should not redeploy Firebase Auth config.
- Firebase Auth configuration remains in `frontend/firebase.json`, but it is now an explicit setup or maintenance command instead of part of the routine `Release` workflow.
- The GitHub Actions deploy service account no longer needs `roles/firebaseauth.admin` because routine releases no longer call `firebase deploy --only auth`.
