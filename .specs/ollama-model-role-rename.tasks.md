# Tasks: Ollama Model Role Rename

## Current State
Implemented. Backend config now uses explicit summary/chat model names, environment-facing names
were renamed to match, and the checked-in docs/examples were updated consistently. `cargo check`,
targeted config tests, and `cargo fmt --check` passed.

## Steps
- [x] Rename backend config fields and environment variable parsing to explicit model-role names.
- [x] Update backend wiring, tests, and startup helpers to use the renamed identifiers.
- [x] Update env examples, deployment/workflow examples, and documentation to match.
- [x] Run targeted verification and record the results.

## Decisions Made During Implementation
- Use `summary_model` and `default_chat_model` in Rust.
- Use `OLLAMA_SUMMARY_MODEL` and `OLLAMA_DEFAULT_CHAT_MODEL` for environment variables.
- Keep behavior unchanged: the default chat model still falls back to the summary model when unset.
- Verification run:
- `cargo check`
- `cargo test from_env_`
- `cargo fmt --check`
