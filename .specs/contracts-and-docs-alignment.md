# Contracts And Docs Alignment

## Problem

The frontend currently mixes hand-authored API DTOs with TS-RS-generated bindings, which creates contract drift risk and duplicates backend-owned types. Documentation and examples are also out of sync with the current codebase in areas such as auth migration status, storage ownership, and model configuration examples.

## Goal

Make generated backend bindings the canonical frontend API contract source and bring documentation and example configuration back into sync with the current system.

## Requirements

- Standardize frontend API modules on generated backend DTO bindings for backend-owned contract types.
- Reduce hand-authored frontend type definitions to UI-only view models and local-only types.
- Define the API module boundary if the current API layer needs to be split for clearer ownership.
- Update stale documentation around auth migration, storage, and model examples.
- Add a validation guardrail that helps prevent contract drift from reappearing.

## Non-Goals

- Replacing TS-RS generation with a different code generation system.
- Rewriting all frontend types, including UI-only presentation models that are not backend contracts.
- Reworking the docs site structure beyond correcting drift and clarifying ownership.

## Design Considerations

- Backend DTO generation already exists, so the lowest-risk improvement is to make it the single source of truth for transport-layer contract types.
- Hand-authored frontend types should remain only where they represent UI composition or derived presentation state.
- Documentation changes should correct current drift without changing the intended architecture unless the code has already established the new truth.

## Open Questions

- None at the moment. The target ownership split between generated DTOs and UI-only types is clear.
