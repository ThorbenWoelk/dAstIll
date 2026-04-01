# Infra Secret Hygiene

## Problem

The repo and Terraform setup still allow patterns that conflict with the intended production security posture. Long-lived service account key material is still provisioned in Terraform, local credential-adjacent artifacts and local Terraform artifacts have been tracked in the repo, and current ignore and validation guardrails do not reliably prevent those files from being committed again.

## Goal

Make infrastructure authentication and repo hygiene consistent with a WIF-only production posture, remove planned long-lived key usage, and define guardrails that prevent sensitive or local-only artifacts from being tracked again.

## Requirements

- Remove long-lived Google service account key creation and related outputs from Terraform design.
- Treat Workload Identity Federation as the only supported production authentication path.
- Define the list of forbidden tracked artifacts and the repo-level validation behavior that should fail when they are present.
- Align ignore rules with the actual local and generated artifacts created by backend, Terraform, and local auth workflows.
- Document the required credential rotation and cleanup follow-up for artifacts that have already been tracked.

## Non-Goals

- Rotating credentials automatically from application code.
- Redesigning the full deployment workflow beyond what is needed to enforce the WIF-only posture.
- Introducing a new secret management product or changing the existing Secret Manager boundary.

## Design Considerations

- The repo already uses WIF in the GitHub Actions deployment path, so the intended target state is already visible and does not require a new auth model.
- Repo hygiene needs both prevention and detection: `.gitignore` alone is insufficient because tracked files remain tracked, so CI validation must also fail on forbidden tracked artifacts.
- Local developer convenience files may still exist locally, but they must not be tracked and should be covered by ignore rules and documentation.

## Open Questions

- None at the moment. The desired target state is clear from the current deployment model and repo guidance.
