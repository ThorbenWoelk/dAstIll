# Tasks: Asynchronous Summary Quality Evaluation

## Current State
Implementation and verification complete. Backend and frontend checks pass locally.

## Steps
- [x] Create spec and task tracking files.
- [x] Add failing backend tests for summary evaluation persistence/reset behavior (Red).
- [x] Implement backend DB/model changes for summary evaluation fields (Green).
- [x] Add async summary evaluation worker using `qwen3-coder:480b-cloud` and persist results.
- [x] Wire API/frontend types and subtly render score + incoherence note in summary UI.
- [x] Run backend/frontend verification gates.

## Decisions Made During Implementation
- Evaluation runs asynchronously and never blocks summary availability.
- Summary quality fields are reset to `NULL` when summary content changes, which re-queues evaluation.
- Worker only evaluates rows where both score and note are `NULL`; if evaluation fails, it stores a short unavailable note to avoid infinite retries.
