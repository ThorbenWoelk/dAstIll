# Tasks: Frontend Controller Boundaries

## Current State
Audit complete. `home-workspace.svelte.ts` and `routes/chat/+page.svelte` remain the largest frontend orchestration hotspots. The main violations are now concrete:

- the home route writes directly into `hw.content.*` and other writable page-controller properties
- the chat route writes directly into controller-owned state and uses two-way bindings against controller setters
- workspace content and sidebar controllers still expose state-bag style writable fields
- a narrower `workspaceContentState/workspaceContentActions` facade already exists, but the raw `content` controller is still publicly exported and used

## Steps
- [x] Create spec and task files for frontend controller boundaries.
- [x] Inventory direct external mutation sites across workspace and chat flows.
- [ ] Freeze the target best-practice rule for this refactor: controller-managed state is readable externally but mutated only through named actions.
- [ ] Decide whether this pass will use a consistent API shape across controllers or only enforce the mutation boundary regardless of shape.
- [ ] Replace home route writes to `hw.content.*`, `hw.mobileBrowseOpen`, `hw.showDeleteAccessPrompt`, `hw.showResetVideoConfirmation`, and `hw.vocabularyModalValue` with named page-controller or workspace-content actions.
- [ ] Stop exporting raw `content` controller access from `createHomeWorkspacePage()` once the route consumes only `workspaceContentState` and `workspaceContentActions`.
- [ ] Replace direct writes in `home-workspace-data-controller.svelte.ts` and `home-workspace-persistence-controller.svelte.ts` with content-controller methods that encapsulate resets, mode changes, and restored state application.
- [ ] Reduce `createContentState()` from a writable state bag to a controller with read-only state plus named mutation methods for mode changes, draft updates, reset flows, and selection transitions.
- [ ] Review `sidebarState` writable fields and decide which must become action-only in this pass versus later follow-up work.
- [ ] Replace chat route writes to `chat.mobileTab`, `chat.draft`, `chat.deepResearch`, and `chat.selectedChatModelId` with named chat-controller actions.
- [ ] Replace controller-facing two-way bindings in the chat route with explicit change handlers or an equivalent action-driven input contract.
- [ ] Verify route components remain render/orchestration shells and no longer coordinate multi-field controller reset sequences directly.
- [ ] Define regression coverage and verification criteria for a behavior-preserving refactor.

## Decisions Made During Implementation
- This refactor is behavior-preserving unless a concrete bug is uncovered.
- Controller-owned state may only be mutated through declared methods after the refactor.
- Route components should orchestrate and render, not own large business-logic blocks.
- The important best practice is a strict mutation boundary, not a required `state`/`actions` nesting convention.
- Existing narrower facades should be preferred over exporting raw controllers when they cover the route's needs.
