# Tasks: Frontend Controller Boundaries

## Current State
Refactor completed. Route/components now read controller state and mutate through named actions, and the remaining low-level selection/video-list setters are kept inside sidebar/controller internals instead of being coordinated from route shells.

## Steps
- [x] Create spec and task files for frontend controller boundaries.
- [x] Inventory direct external mutation sites across workspace and chat flows.
- [x] Freeze the target best-practice rule for this refactor: controller-managed state is readable externally but mutated only through named actions.
- [x] Decide whether this pass will use a consistent API shape across controllers or only enforce the mutation boundary regardless of shape.
- [x] Replace home route writes to `hw.content.*`, `hw.mobileBrowseOpen`, `hw.showDeleteAccessPrompt`, `hw.showResetVideoConfirmation`, and `hw.vocabularyModalValue` with named page-controller or workspace-content actions.
- [x] Stop exporting raw `content` controller access from `createHomeWorkspacePage()` once the route consumes only `workspaceContentState` and `workspaceContentActions`.
- [x] Replace direct writes in `home-workspace-data-controller.svelte.ts` and `home-workspace-persistence-controller.svelte.ts` with content-controller methods that encapsulate resets, mode changes, and restored state application.
- [x] Reduce `createContentState()` from a writable state bag to a controller with read-only state plus named mutation methods for mode changes, draft updates, reset flows, and selection transitions.
- [x] Review `sidebarState` writable fields and decide which must become action-only in this pass versus later follow-up work.
- [x] Replace chat route writes to `chat.mobileTab`, `chat.draft`, `chat.deepResearch`, and `chat.selectedChatModelId` with named chat-controller actions.
- [x] Replace controller-facing two-way bindings in the chat route with explicit change handlers or an equivalent action-driven input contract.
- [x] Verify route components remain render/orchestration shells and no longer coordinate multi-field controller reset sequences directly.
- [x] Define regression coverage and verification criteria for a behavior-preserving refactor.

## Decisions Made During Implementation
- This refactor is behavior-preserving unless a concrete bug is uncovered.
- Controller-owned state may only be mutated through declared methods after the refactor.
- Route components should orchestrate and render, not own large business-logic blocks.
- The important best practice is a strict mutation boundary, not a required `state`/`actions` nesting convention.
- Existing narrower facades should be preferred over exporting raw controllers when they cover the route's needs.
