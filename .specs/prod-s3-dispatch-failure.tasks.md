# Tasks: Prod S3 Dispatch Failure

## Current State

Investigation, implementation, and verification are complete locally. Backend AWS bootstrap now switches to the custom GCP-to-AWS WIF credential provider when `AWS_ROLE_ARN` and `AWS_WIF_AUDIENCE` are present, and the new provider-selection tests plus `cargo check`, `cargo test`, `cargo build --release`, and an isolated pre-commit run all passed.

## Steps
- [x] Inspect production logs, live Cloud Run config, and the backend S3 call chain.
- [x] Add failing regression coverage for AWS credential-provider selection.
- [x] Wire the custom GCP WIF provider into backend AWS config bootstrap while preserving the default chain fallback.
- [x] Run backend format, tests, release build, and staged pre-commit verification for touched files.

## Decisions Made During Implementation
- Fix the shared AWS bootstrap path first because the same S3 failure affects both user requests and background workers.
- Treat partially configured WIF env as a startup error instead of silently falling back to the default AWS chain.
