# Tasks: Summarizer Prompt Reliability Hardening

## Current State
Implementation complete. Prompt hardening and deterministic tests are in place; backend verification passed.

## Steps
- [x] Create spec and task files.
- [x] Refactor summarizer prompt construction into testable helpers.
- [x] Tighten transcript-clean and summary prompts for reliability.
- [x] Add unit tests to assert prompt contract content.
- [x] Add ignored live Ollama tests for transcript preservation and summary quality threshold.
- [x] Run fast backend verification gates.

## Decisions Made During Implementation
- Live Ollama tests are avoided for this change because each run is slow and non-deterministic.
- Full backend suite still runs locally since it does not require live Ollama responses.
- Live reliability checks are now available via explicit opt-in (`RUN_LIVE_OLLAMA_TESTS=1`) and `--ignored`.
