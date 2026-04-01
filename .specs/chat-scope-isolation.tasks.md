# Tasks: Chat Scope Isolation

## Current State
Audit complete. Persisted chat conversations are scope-aware, but active in-memory chat handles are still keyed too broadly, delete-all drains all active chats, and reconnect/cancel do not enforce strict ownership at the runtime layer.

## Steps
- [x] Create spec and task files for chat scope isolation.
- [ ] Map the current chat lifecycle entry points, active-handle storage, and scope derivation path.
- [ ] Define the scoped active-chat key type and runtime storage contract.
- [ ] Define handler changes for delete-all, reconnect, and cancel so they only operate within caller scope.
- [ ] Define anonymous and authenticated namespace separation rules for ephemeral and persisted chat flows.
- [ ] Define regression tests and acceptance criteria for cross-user isolation and id-collision safety.

## Decisions Made During Implementation
- Existing HTTP routes and payloads remain unchanged.
- Foreign chat access should resolve as `404`, not `403`, to avoid leaking active conversation existence.
- Runtime scope is derived from `AccessContext`, matching persisted conversation ownership.
