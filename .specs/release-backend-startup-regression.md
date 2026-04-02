# Release Backend Startup Regression

## Problem

`Validation` is green, but `Release` fails while deploying the backend Cloud Run revision. The service never becomes healthy because the deploy workflow corrupts at least one runtime environment variable before the process binds `PORT`.

## Goal

Release the backend with the same effective runtime configuration that works locally and in prior successful revisions, without mangling comma-containing env values during deployment.

## Requirements

- The `Release` workflow must deploy backend runtime env vars without splitting comma-containing values into separate variables.
- The backend Cloud Run revision must be able to start with logfire enabled and bind `PORT`.
- The workflow change must preserve authoritative env var deployment semantics.
- The fix should also protect other comma-containing env values, such as multi-origin CORS lists.

## Non-Goals

- Changing backend startup behavior or relaxing required runtime config validation.
- Changing secret sourcing or Terraform-managed infrastructure.
- Broad release workflow refactors unrelated to env var transport.

## Design Considerations

- The deploy action already supports `env_vars_file`, which avoids delimiter parsing issues that affect inline `env_vars`.
- Using the prepared env file directly is more robust than trying to escape individual comma-containing values one by one.

## Open Questions

- None at the moment. The failing revision logs and Cloud Run service config point to a concrete workflow regression.
