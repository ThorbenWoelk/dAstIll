# Tasks: Ollama Indicator Status

## Current State
Backend and frontend expose explicit AI status states, and the indicator is now clickable with a detail popover on both workspace and queue pages. Frontend verification passed locally again after the interaction change (`bun run format:check`, `bun test`, `bun run check`, `bun run build`).

## Steps
- [x] Create spec and task tracking files.
- [x] Write failing tests for backend AI status classification and frontend status handling.
- [x] Implement backend AI status payloads.
- [x] Update workspace and queue indicators to use the new status.
- [x] Run format, lint/check, tests, and release build verification.

## Decisions Made During Implementation
- Preserve the existing boolean availability fields for action enablement and backward-compatible gating.
- Drive the color change from an explicit backend `ai_status` field instead of inferring it in the UI.
- Treat cloud cooldown without a usable local fallback as `offline`, since the indicator should not imply working local generation when no local path is configured.
- Fix pre-existing clippy blockers in cooldown helpers and YouTube continuation parsing so the required backend lint gate remains green.
- Use a shared clickable status component instead of hover-only tooltip behavior so the state explanation remains accessible on touch devices.
