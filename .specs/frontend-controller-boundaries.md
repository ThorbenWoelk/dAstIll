# Frontend Controller Boundaries

## Problem

Large frontend controllers and route components have accumulated too much orchestration and mutable state, especially in the main workspace controller and chat route. The codebase has started introducing controllers, but several routes and sibling controllers still write directly into controller-owned state through exposed setters and two-way bindings.

That leaves key frontend flows in an in-between state:

- route components still mutate controller internals
- controller modules still expose state-bag style writable properties
- stricter state/action facades exist in some places but remain optional because raw controllers are also exported

This weakens ownership boundaries, spreads reset and synchronization sequences across call sites, and makes behavior-preserving cleanup harder because invariants are enforced by convention instead of API shape.

## Goal

Refactor the largest frontend controllers and route-level orchestrators into smaller focused modules while enforcing an explicit controller mutation boundary: routes and components may read controller-managed state, but they must change it only through named controller actions.

## Requirements

- Split the main workspace controller into smaller focused controller modules where responsibilities are currently mixed.
- Keep chat page orchestration in a dedicated controller module rather than route-local mutation logic.
- Replace direct route/component writes to controller-owned UI state with named controller actions.
- Replace two-way bindings that write directly into controller-owned state with explicit change callbacks or equivalent action-driven wiring.
- Remove public writable controller fields for controller-owned domain/UI state where those writes should be action-only.
- Consolidate repeated reset and synchronization sequences into controller-owned methods so callers do not manually coordinate multiple state fields.
- Preserve existing user-visible behavior, URL/route contracts, and existing component responsibilities unless a concrete bug requires a narrow behavior fix.
- Make the resulting controller boundary verifiable in code review:
  - routes/components read controller state
  - routes/components invoke named actions to change controller state
  - controller-owned invariants are centralized inside controller methods

## Non-Goals

- Reworking the visual design or route structure.
- Rewriting the workspace or chat experiences from scratch.
- Introducing new product behavior unless required to fix a bug found during the refactor.
- Mandating a specific API nesting shape such as `state`/`actions` if a flatter read-plus-action API achieves the same mutation boundary.
- Refactoring every frontend state holder in the repo during this pass if a controller is already internal-only and not bypassing ownership boundaries.

## Design Considerations

- This pass should be behavior-preserving and prioritize clearer ownership over new abstractions.
- Route components should become orchestration and rendering shells rather than business-logic containers.
- Controller state should be mutated through declared methods so side effects remain centralized and testable.
- The important boundary is not object nesting. `controller.mobileTab` is acceptable to read if mutation still happens through named actions such as `controller.openConversations()` or `controller.closeConversations()`.
- The main current hotspots are:
  - workspace route writes into `hw.content.*` and other writable page-controller fields
  - chat route writes into `chat.mobileTab`, `chat.draft`, `chat.deepResearch`, and `chat.selectedChatModelId`
  - workspace content and sidebar controllers expose writable state-bag surfaces that encourage external resets
- Prefer deletion over addition when introducing mutation APIs:
  - remove raw setters once a named action exists
  - stop exporting raw controllers when a narrower facade already covers the route's needs
- Follow the repo's Svelte 5 state guidance: preserve reactive boundaries cleanly and avoid leaking writable `$state` ownership across module boundaries.

## Open Questions

- Should the cleanup standardize on a shared controller API shape across workspace and chat, or is consistency at the mutation-boundary level sufficient for this pass?
- Are there any intentional exceptions where a writable setter should remain public, or should controller-owned UI/domain state become action-only by default?
