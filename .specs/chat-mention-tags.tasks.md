# Tasks: Chat Mention Tags

## Current State
Shared mention parsing, resolution, composer tags, and user-message tag rendering are implemented. Targeted tests pass; full `svelte-check` is currently blocked by unrelated existing frontend errors outside the chat files.

## Steps
- [x] Define the UI behavior and implementation approach
- [x] Add a shared frontend mention parsing and resolution utility
- [x] Add a reusable minimal chat mention tag component
- [x] Render resolved mention tags in the composer for both `@{...}` and `+{...}`
- [x] Render resolved mention tags inline in user message bubbles
- [x] Add unit tests for mention parsing and exact-match resolution helpers
- [ ] Run frontend verification and repo pre-commit checks

## Decisions Made During Implementation
- Keep the stored text unchanged and only change presentation.
- Reuse the existing suggestion endpoints for mention resolution instead of adding a new resolver API.
- The composer shows mention tags inside the chat box as a compact tag row above the textarea, rather than trying to turn the textarea into a rich text editor.
