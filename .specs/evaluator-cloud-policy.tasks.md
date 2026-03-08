# Tasks: Evaluator Cloud Policy

## Current State
Policy enforced. Summary evaluation now validates `SUMMARY_EVALUATOR_MODEL` at startup, only allowing cloud models with parseable parameter sizes above 40B. Local evaluator fallback is no longer configured, and backend verification passed locally.

## Steps
- [x] Create spec and task tracking files.
- [x] Write failing tests for evaluator model validation.
- [x] Enforce evaluator startup validation and remove local fallback usage.
- [x] Run backend verification gates.

## Decisions Made During Implementation
- Validation will be deterministic from the configured model name rather than a remote capability probe.
- Cloud-model validation accepts both `:cloud` and `-cloud` naming styles, then enforces a parsed parameter token like `70b` or `480b`.
- The evaluator startup path warns and ignores `SUMMARY_EVALUATOR_FALLBACK_MODEL` because evaluation is explicitly cloud-only.
