# Tasks: Chat Runtime Scope Isolation

## Current State
Implemented. Active chat runtime keys now include scope ownership, reconnect and cancel are scope-aware, delete-all only removes active chats from the caller's scope, and targeted backend formatting plus chat handler tests and `cargo check` passed.

## Steps
- [x] Create spec and task files for chat runtime scope isolation.
- [x] Add regression coverage for foreign-scope reconnect/cancel, scoped delete-all, and anonymous/authenticated id collisions.
- [x] Scope active runtime chat handles by ownership scope plus conversation id.
- [x] Run targeted verification for the backend chat handlers and runtime cleanup paths.

## Decisions Made During Implementation
- Runtime ownership will use an explicit composite active-chat key instead of overloading raw conversation ids.
- The runtime scope key uses `AccessContext.cache_scope_key()` so authenticated and anonymous scopes cannot alias even if a user id string matches `anonymous`.
- Scoped delete-all now removes matching active handles from the registry before cancellation so foreign scopes cannot reconnect to or cancel those handles during teardown.
