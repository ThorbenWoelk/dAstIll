# Tasks: Transcript Clean Formatting Action

## Current State
Implemented and verified with backend/frontend checks plus live API smoke test.

## Steps
- [x] Add backend test coverage for transcript text-preservation comparison logic (Red).
- [x] Implement Ollama-backed transcript formatting method with preservation guard (Green).
- [x] Add transcript formatting API endpoint and wire route.
- [x] Add minimal icon action in transcript editor and integrate frontend API call.
- [x] Run format, lint/check, and tests for backend and frontend.

## Decisions Made During Implementation
- Transcript formatting endpoint always returns text that preserves input wording: if Ollama output changes token sequence, backend returns original input and marks `preserved_text=false`.
- Equality guard compares whitespace-token sequence, allowing only spacing/line-break changes.
- Clean-format icon is visible in transcript read mode and edit mode; clicking in read mode transitions into edit mode with cleaned draft text for explicit save/cancel.
- Backend now logs transcript-clean requests and Ollama prompt lifecycle (operation, model, timing, and safety fallback) for observability.
- Frontend formatting status is now scoped to the target video id to avoid stale "Formatting…" messages after switching videos.
- Formatter prompt now requests logical section headings plus `<mark>` highlights for key message phrases.
- Text-preservation guard now allows markdown headings and inline markup tags while enforcing unchanged transcript body token order.
- Transcript clean now runs a self-healing loop with up to 3 Ollama attempts; on each failed attempt it feeds mismatch diagnostics (reason, token index, expected vs actual context) back into the next prompt.
