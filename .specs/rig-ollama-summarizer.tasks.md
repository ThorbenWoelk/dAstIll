# Tasks: Rig Ollama Summarizer Migration

## Current State
Migration completed. Summaries now use a `rig` Ollama agent, and backend verification gates (`fmt`, `clippy -D warnings`, `test`, `build --release`) passed locally.

## Steps
- [x] Create migration spec and task tracking files.
- [x] Replace direct summarize HTTP call with `rig` Ollama agent call.
- [x] Keep `OLLAMA_URL` compatibility for rig base URL configuration.
- [x] Add/update tests for migrated summarizer behavior.
- [x] Run format, lint, and tests; capture verification results.

## Decisions Made During Implementation
- Keep `is_available()` as a direct `/api/tags` health probe using existing shared `reqwest` client.
- Keep the summary prompt semantics unchanged while changing provider plumbing.
- Build Ollama client with `ollama::Client::builder().base_url(OLLAMA_URL)` instead of mutating global env vars for `from_env()`.
