# Tasks: Infra Secret Hygiene

## Current State
Audit complete. Terraform still provisions a long-lived backend service account key, the repo currently tracks local credential-adjacent and Terraform state/plan artifacts, and current ignore rules do not cover all generated or sensitive local files.

## Steps
- [x] Create spec and task files for infra secret hygiene.
- [ ] Inventory all currently tracked sensitive, credential-adjacent, and local-state artifacts that violate the target posture.
- [ ] Define the Terraform changes that remove the backend service-account-key strategy and related outputs.
- [ ] Define `.gitignore` additions and the forbidden-artifact validation guardrail for CI and local hooks.
- [ ] Define the credential rotation and repo cleanup checklist for already tracked artifacts.
- [ ] Define verification steps and acceptance criteria for the WIF-only production posture.

## Decisions Made During Implementation
- Production authentication remains WIF-only.
- No fallback to long-lived Google service account JSON keys will remain in the planned infrastructure design.
- CI should fail when forbidden tracked artifacts are present, not just rely on ignore rules.
