# Tasks: Chat Delete All

## Current State
Bulk delete is implemented across the chat API and sidebar UI. Unit coverage, headed Playwright verification, frontend build/check, backend tests, backend release build, and the staged pre-commit hook all passed.

## Steps
- [x] Create spec and task files for chat delete-all.
- [x] Add failing automated coverage for delete-all behavior.
- [x] Implement backend bulk delete support.
- [x] Implement chat UI delete-all affordance and confirmation flow.
- [x] Run verification commands, manual validation, and record outcomes.

## Decisions Made During Implementation
- Bulk delete will live in the chat sidebar and use the existing confirmation modal pattern.
- The initial implementation target is a dedicated backend delete-all endpoint so the client can issue one destructive request.
- The confirmation modal now serves both single-conversation delete and delete-all flows, with copy derived from the active delete target.
- Headed Playwright coverage seeds and clears chat conversations through the existing chat API so the UI workflow stays reproducible against a live local stack.
