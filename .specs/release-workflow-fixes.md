# Architecture Decision Record

# Status

Accepted

# Context

The latest `Release` workflow on `main` failed on April 1, 2026 in two deployment jobs:

- `Deploy Backend` pushed blank renamed Ollama model env vars into Cloud Run after the workflow moved from `OLLAMA_MODEL` to split role-specific vars. The deploy payload also embedded a literal comment line inside the multiline `env_vars` value.
- `Deploy Firebase Auth` ran with the GitHub Actions deploy service account, but Terraform had not granted that identity the Firebase Auth admin permissions needed by `firebase deploy --only auth`.

# Decision

Harden the release path in-repo by:

- Making backend deploys backward-compatible with the legacy `OLLAMA_MODEL` GitHub variable until the split vars are fully populated.
- Removing inline comments from the `env_vars` payload passed to `deploy-cloudrun`.
- Granting the GitHub Actions deploy service account `roles/firebaseauth.admin` through Terraform.
- Updating deployment documentation to reflect the deploy identity requirements.

# Consequences

- Releases stay compatible with the existing GitHub variable set while the renamed Ollama vars roll out.
- Backend deploys stop publishing invalid or empty runtime configuration to Cloud Run.
- Firebase Auth deploys can manage auth config without manual console intervention.
- The GitHub Actions service account gains one additional project-level admin role scoped to Firebase Auth operations.
