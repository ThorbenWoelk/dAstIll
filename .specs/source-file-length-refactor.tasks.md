# Tasks: Source file length refactor

## Current State
Third wave: `chat/+page.svelte` split into focused components and `$lib/chat` helpers. `sidebar-state` and further `chat/mod.rs` splits still optional.

## Steps
- [x] Re-run line-count audit on `frontend/src` and `backend/src` (exclude `node_modules`, `target`, `.svelte-kit`, build outputs) and update the table in `source-file-length-refactor.md` if top offenders shifted.
- [x] Split `frontend/src/routes/+page.svelte`: extract self-contained UI sections and pure TS into `$lib`; keep route as composition. Verify with `bun run check` and targeted tests.
- [x] Split `backend/src/services/chat` along natural module boundaries; update `mod` tree and imports. Verify with `cargo check` and tests touching chat. (Follow-up: further split `chat/mod.rs` service implementation.)
- [x] Split `frontend/src/lib/components/workspace/WorkspaceSidebar.svelte` into smaller components or extracted logic. Verify workspace navigation and keyboard shortcuts still work. (Follow-up: channel list / preview sections.)
- [x] Split `frontend/src/routes/chat/+page.svelte` similarly to the home route pattern. Verify chat flows.
- [ ] Evaluate `frontend/src/lib/workspace/sidebar-state.svelte.ts` for extraction by concern; split only where boundaries are clear. Verify sidebar behavior and existing tests.
- [x] Split `backend/src/services/summarizer.rs` if it mixes unrelated responsibilities; verify summarization paths still compile and behave as before.
- [ ] Triage the 500–900 line tier: list 3–5 next candidates and schedule incremental extractions (optional, after critical tier).
- [ ] Decide on optional policy: document a recommended max lines per file type and whether to add CI or a script (resolve Open Questions in spec).

## Decisions Made During Implementation
- **Home route**: Desktop workspace header (content mode tabs, `ContentEditor`, search slot) lives in `WorkspaceDesktopTopBar.svelte`; mobile browse overlay is `WorkspaceMobileBrowseOverlay.svelte`.
- **Summary session id**: `hashSummarySignature` / `deriveSummaryTrackingId` moved to `frontend/src/lib/workspace/summary-tracking-id.ts` with `frontend/tests/summary-tracking-id.test.ts`.
- **Summarizer**: Replaced single `summarizer.rs` with `services/summarizer/mod.rs` (service + tests), `prompts.rs` (preambles + prompt builders), `transcript_compare.rs` (token equivalence and mismatch helpers).
- **Dead imports**: Removed unused `ContentEditor` and `$lib/workspace/navigation` imports from `+page.svelte` (were unused before this change).
- **Workspace sidebar**: Video filter popover + positioning live in `WorkspaceSidebarVideoFilterControl.svelte`; shared option labels in `sidebar-filter-options.ts`.
- **Chat service**: `services/chat/mod.rs` replaces `chat.rs`; limits and prompts in `constants.rs`; `ChatQueryIntent` in `intent.rs`. Use `pub(crate) use constants::*` so `pub(crate)` items re-export correctly to sibling `services/*` modules.
- **Chat route UI**: `ChatMobileConversationsOverlay`, `ChatContentSectionHeader`, `ChatConversationMeta`, `ChatAnonymousQuotaNotice`; helpers `starter-prompts.ts`, `anonymous-quota.ts`, `conversation-meta.ts` (`ChatStreamTiming`).
