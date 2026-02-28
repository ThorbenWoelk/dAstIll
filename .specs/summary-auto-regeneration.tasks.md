# Tasks: Automatic Summary Regeneration for Low Quality

## Current State
Implementation complete. Backend verification gates pass locally.

## Steps
- [x] Create spec and task tracking files.
- [x] Add failing backend tests for retry counter and regeneration decision logic (Red).
- [x] Implement DB + worker/content orchestration for auto-regeneration (Green).
- [x] Refactor and ensure code clarity.
- [x] Run verification gates (fmt, clippy, test, release build).

## Decisions Made During Implementation
- Regeneration threshold: score `< 7`.
- Retry policy: bounded automatic retries to avoid infinite loops.
- Retry limit chosen: `2` automatic regeneration attempts per summary revision.
- Manual summary edits reset auto-regeneration attempts and clear quality fields.
