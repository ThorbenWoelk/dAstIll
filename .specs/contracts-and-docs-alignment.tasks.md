# Tasks: Contracts And Docs Alignment

## Current State
Docs have been corrected for the current Firebase auth model, user-scoped storage layout, and embedding-model configuration wording. Frontend contract ownership cleanup is still pending.

## Steps
- [x] Create spec and task files for contracts and docs alignment.
- [ ] Inventory duplicated DTOs and mixed generated/manual import sites across the frontend API layer.
- [ ] Define the canonical type ownership rules between generated bindings and UI-only frontend types.
- [ ] Define the API module boundary changes needed to support cleaner contract ownership.
- [x] Define the required documentation and example corrections for auth, storage, and model configuration.
- [ ] Define validation and verification guardrails that prevent future contract drift.

## Decisions Made During Implementation
- TS-RS-generated bindings are the canonical source for backend-owned DTOs.
- Hand-authored frontend types may remain only for UI-only or derived presentation state.
- Documentation must reflect the current completed auth migration state and valid, non-floating model examples.
- Architecture docs should describe stored-record ownership separately from API view-model overlays when user-scoped state is merged into responses.
