# ADR: Secret Values Live In Secret Manager, Not Terraform State

## Status

Accepted

## Context

Production secrets were previously created and versioned through Terraform variables.
That made Terraform state a holder of application secret payloads and tied secret rotation
to local `terraform.tfvars` handling.

This repo is worked from multiple local worktrees and deployed through GitHub Actions.
That setup benefits from shared Terraform state and CI-driven infra ordering, but it should
not move application secret values into CI variables or leave them in Terraform state.

We also need frontend Firebase build config (`apiKey`, `authDomain`) available to CI after
Terraform creates or updates the Firebase web app.

The repo manages GCP resources through Terraform, so CI uses short-lived GCP credentials through
GitHub -> GCP federation.

## Decision

Terraform manages only:

- Secret Manager secret containers
- IAM bindings for runtime and CI identities
- Other infrastructure resources

Terraform does not manage application secret versions.

Secret payloads are written directly to Secret Manager as new versions.
Rotation happens by adding a new Secret Manager version and redeploying the consumer.

The dedicated infra workflow runs before app deployment when Terraform changes land on the
same push. After Terraform apply, that workflow reads Firebase web app config from the
Firebase Management API and syncs only the frontend build secrets into Secret Manager.

CI authenticates to GCP through GitHub OIDC and the dedicated Terraform service account.

## Consequences

Positive:

- removes app secret payloads from Terraform state
- keeps secret lifecycle boundaries clear: Terraform owns containers and IAM, Secret Manager owns values
- works cleanly with separate worktrees because infra ordering moves to CI, not local operator timing
- keeps frontend Firebase build config current without reintroducing Terraform-state secret storage
- gives Terraform in CI a distinct GCP trust path without long-lived service account keys

Tradeoffs:

- non-Firebase app secrets now need an explicit Secret Manager version add during bootstrap or rotation
- infra CI must fail if required secret versions are missing instead of silently proceeding
- secret deprecation remains an IaC chore because containers and IAM must still be retired through Terraform
- first creation of the GCP Workload Identity Federation resources still requires one authenticated bootstrap apply

## Directive

Do not reintroduce application secret payloads into Terraform variables, Terraform state,
or GitHub repository variables.

When retiring a secret, remove consumers first, then remove workflow references, IAM bindings,
and Terraform secret resources in the same planned IaC change or in an explicit staged follow-up.

Keep GitHub CI access and Cloud Run runtime access as separate service accounts. Do not collapse
them back into one trust boundary.
