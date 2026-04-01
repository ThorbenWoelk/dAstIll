# Tasks: Frontend Controller Boundaries

## Current State
Audit complete. `home-workspace.svelte.ts` and `routes/chat/+page.svelte` remain the largest frontend orchestration hotspots, and multiple routes/components still write directly into controller-owned state instead of going through explicit mutators.

## Steps
- [x] Create spec and task files for frontend controller boundaries.
- [ ] Inventory direct external mutation sites across workspace and chat flows.
- [ ] Define target module boundaries for the workspace controller split.
- [ ] Define the extraction plan for chat page orchestration into a dedicated controller module.
- [ ] Define explicit mutator APIs that replace direct controller-owned state writes.
- [ ] Define regression coverage and verification criteria for a behavior-preserving refactor.

## Decisions Made During Implementation
- This refactor is behavior-preserving unless a concrete bug is uncovered.
- Controller-owned state may only be mutated through declared methods after the refactor.
- Route components should orchestrate and render, not own large business-logic blocks.
