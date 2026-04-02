# Environment Config Hygiene

## Problem

Local `.env` files, Terraform-managed secrets, GitHub deployment configuration, deployed Cloud Run services, and Secret Manager can drift over time. That creates stale environment variables, duplicate aliases, and unused secrets that increase operational risk and make deployments harder to reason about.

## Goal

Establish the current source of truth for runtime configuration, identify stale or obsolete environment variables across local and production surfaces, and remove or document them so the deployed setup matches the app's actual requirements.

## Requirements

- Inventory local `.env` files and the environment variables currently referenced by backend, frontend, docs, Terraform, and deployment workflow code.
- Inventory deployed Cloud Run environment variables and secret mounts for each production service without exposing secret values.
- Inventory Secret Manager secrets relevant to the application and determine which are actively referenced versus stale.
- Remove or update repo-managed configuration that still carries obsolete variables, aliases, or secret references.
- Clean up safe-to-remove stale runtime configuration in Cloud Run and Secret Manager when ownership is clear from the audit.
- Document any ambiguous or intentionally retained variables that should not be deleted yet.

## Non-Goals

- Rotating secret values that are still in active use.
- Redesigning the deployment architecture beyond configuration hygiene.
- Changing application behavior unrelated to env/config cleanup.

## Design Considerations

- The cleanup should be usage-driven: code references and deployment definitions determine whether a variable is still needed.
- Secret values must remain unread and unprinted throughout the audit.
- Remote deletions should only happen when a variable or secret is clearly unused and not Terraform-managed for future applies.

## Open Questions

- Whether any legacy aliases should remain temporarily for backward compatibility with older revisions or operational scripts.
