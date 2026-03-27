# Prod S3 Dispatch Failure

## Problem

Production backend requests that need summary, transcript, or search-index data are failing with `S3 error: dispatch failure`, surfacing to users as generic database errors. Cloud Run injects AWS WIF environment variables, but the backend currently builds AWS clients from the default credential chain and never uses the repo's custom GCP-to-AWS WIF provider.

## Goal

Production AWS-backed storage access should authenticate through the intended GCP-to-AWS WIF flow so S3 and S3 Vectors requests stop failing with transport-level dispatch errors.

## Requirements

- Backend AWS client bootstrap must use the custom GCP WIF credential provider when the production WIF environment variables are present.
- Backend startup must keep working for local development and other environments that rely on the default AWS credential chain.
- The provider-selection logic must be covered by automated tests so the production wiring cannot silently regress.
- Backend verification must include tests and a release build for the touched code.

## Non-Goals

- Reworking how summaries or transcripts are stored.
- Changing Terraform or Cloud Run environment variable names in this pass.
- Refactoring the summary route's duplicate S3 reads unless required by the auth fix.

## Design Considerations

- Keep the fix at AWS config bootstrap, because workers and multiple request paths all fail on S3 access and share the same client construction.
- Detect the WIF path from existing `AWS_ROLE_ARN` and `AWS_WIF_AUDIENCE` env vars so the runtime matches the deployed infrastructure contract.
- Preserve the default AWS chain as a fallback for local tools, tests, and any non-Cloud-Run usage.

## Open Questions

- None for the initial fix. If production still reports `dispatch failure` after the WIF provider is wired, the next step is deeper transport-level logging for AWS SDK errors.
