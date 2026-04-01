# Frontend Controller Boundaries

## Problem

Large frontend controllers and route components have accumulated too much orchestration and mutable state, especially in the main workspace controller and chat route. Some route and component code also bypasses controller-owned mutation APIs and writes directly into controller internals, which makes side effects harder to reason about and violates the repo's Svelte state rules.

## Goal

Refactor the largest frontend controllers and route-level orchestrators into smaller focused modules while enforcing explicit mutation boundaries for controller-owned state.

## Requirements

- Split the main workspace controller into smaller focused controller modules.
- Extract non-render chat page logic out of the chat route component into a dedicated controller module.
- Add explicit mutator methods where controller state is currently written externally.
- Remove direct external writes to controller-owned state.
- Preserve existing user-visible behavior and route contracts.

## Non-Goals

- Reworking the visual design or route structure.
- Rewriting the workspace or chat experiences from scratch.
- Introducing new product behavior unless required to fix a bug found during the refactor.

## Design Considerations

- This pass should be behavior-preserving and prioritize clearer ownership over new abstractions.
- Route components should become orchestration and rendering shells rather than business-logic containers.
- Controller state should be mutated through declared methods so side effects remain centralized and testable.

## Open Questions

- None at the moment. The main refactor targets and desired ownership model are clear.
