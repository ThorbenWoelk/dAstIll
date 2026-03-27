# Tasks: Channel Video List Performance

## Current State
Selected-channel bootstrap seeding, paged sidebar expansion, and backend channel page queries are implemented. Targeted frontend tests and backend library tests are passing; remaining verification is frontend build/pre-commit, with a known unrelated `svelte-check` parser error in `frontend/src/lib/components/chat/ChatInput.svelte`.

## Steps
- [x] Patch backend payloads and data access for paged channel snapshot/list loading.
- [x] Patch shared frontend bootstrap, API, sidebar state, and channel route hydration.
- [x] Add or update backend/frontend tests for bootstrap seeding, paging, and sidebar behavior.
- [ ] Run format, lint/check, tests, build, and staged pre-commit verification.

## Decisions Made During Implementation
- Use the existing workspace bootstrap endpoint to seed the selected channel preview instead of introducing a new API.
- Keep `__others__` on the existing scan-backed path in this pass.
- Keep expanded per-channel infinite loading scroll-driven in the sidebar, with a manual `Load more` fallback, instead of adding a new virtualization/paging dependency.
- Leave the existing unrelated chat parser error outside this change; it blocks repo-wide `bun run check` but not the targeted sidebar/channel performance tests.
