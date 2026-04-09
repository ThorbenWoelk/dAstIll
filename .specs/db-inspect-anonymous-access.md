# db_inspect Anonymous Access

## Context
- **Problem**: The `db_inspect` tool is denied to anonymous users, preventing them from browsing stored library metadata (videos, channels, transcripts, summaries) - data that is already publicly visible in the UI. The restriction creates an inconsistent UX where anonymous users can see content in the app but cannot query it via the AI assistant.
- **Goal**: Allow anonymous users to use `db_inspect` for read-only queries against library data (videos, channels, summaries, transcripts) they already have access to through the "Other" channel and public content.
- **Linear**: N/A

## Implementation Plan
- [ ] **Phase 1: Audit access boundaries.** Map what data anonymous users can already see (UI, API endpoints). Confirm `db_inspect` operations are read-only and safe to expose.
- [ ] **Phase 2: Lift the auth restriction.** Change `can_use_db_inspect()` to permit `AuthState::Anonymous`. Keep Operator-only restrictions for any destructive operations (none exist currently, but document this invariant).
- [ ] **Phase 3: Scope query results.** Ensure `db_inspect` results filter by `allowed_channel_ids` and `allowed_other_video_ids` just like search and retrieval paths. Anonymous users should see the same subset of data they can see elsewhere.
- [ ] **Phase 4: Update LLM planner hints.** Remove the "Do not use `db_inspect` unless the caller is signed in" instruction from the planner context (`helpers.rs:155`). Update to "Only use `db_inspect` for read-only library queries."
- [ ] **Phase 5: Update error messaging.** Replace the forbidden result message "Database inspection requires a signed-in session." with a scoped alternative if a future operation type needs auth (not needed in this PR, but keep the hook available).

## Requirements
- [ ] **Requirement 1**: Anonymous users can successfully call `db_inspect` with `count`, `list`, and `breakdown` operations. -> Verification: Unit test asserting `can_use_db_inspect()` returns `true` for `AuthState::Anonymous` with valid `AccessContext`.
- [ ] **Requirement 2**: `db_inspect` results are filtered by the caller's access scope (`allowed_channel_ids`, `allowed_other_video_ids`). -> Verification: Integration test shows anonymous user with seeded channel access can query seeded channel data but not private channels.
- [ ] **Requirement 3**: LLM planner instructions do not block `db_inspect` for anonymous callers. -> Verification: Check `format_access_context_hint()` output for anonymous users no longer contains the authentication restriction line.
- [ ] **Requirement 4**: No regression for authenticated users. -> Verification: Existing authenticated access continues to work; all current tests pass.

## Verification Gates
- [ ] **TDD**: Write red test for anonymous `db_inspect` access. Write red test for access-scoped results. Run all tests after changes.
- [ ] **CSO**: STRIDE audit for information disclosure. Confirm no data leaks beyond existing UI/API exposure. Verify operation is read-only.
- [ ] **Design Review**: Ensure error messages and planner hints remain helpful.
- [ ] **Success**: Evidence (logs, test output, manual verification) provided showing anonymous queries work and are properly scoped.

## Anti-Rationalization (Blocked Excuses)
- "Anonymous users shouldn't query the database directly." — They already can via search and the UI. `db_inspect` is just another read path for the same data.
- "We need rate limiting first." — Rate limiting applies to all users. Implement orthogonally, not as a blocker.
- "Just remove auth and ship it." — Must also scope results correctly. Lifting the restriction without access filtering would expose private channels to anonymous users.

## Technical Notes

### Current Restriction (backend/src/security.rs:210-212)
```rust
pub fn can_use_db_inspect(access_context: &AccessContext) -> bool {
    access_context.auth_state.is_authenticated()
}
```

### Enforcement Points
1. **Tool loop execution** (`backend/src/services/chat/tool_loop.rs:188-194`)
2. **LLM planner hint** (`backend/src/services/chat/helpers.rs:155`)
3. **Forbidden result** (`backend/src/services/chat/tools/queries.rs:524-528`)

### Access Scope Pattern
The `AccessContext` already contains:
- `allowed_channel_ids`: channels the user can see
- `allowed_other_video_ids`: specific videos in the "Other" virtual channel

All `db_inspect` query functions (`execute_db_inspect_query`) should respect these scopes. This is already done for search/retrieval via `filter_search_candidates_for_access`.

### Security Model
- **Data model**: Videos belong to channels. Users have subscriptions to channels. Anonymous users see the seeded "Other" channel and any public videos.
- **Read path guarantee**: `db_inspect` operations are `count`, `list`, `breakdown`. All are read-only. No `insert`, `update`, or `delete` paths exist.
- **Disclosure boundary**: If a video/channel is visible in the UI (`can_access_video`, `can_access_channel`), it should be queryable via `db_inspect`. If not visible, `db_inspect` must also exclude it.

## Non-Goals
- Adding write operations to `db_inspect` (out of scope).
- Changing anonymous user permissions for channels/videos they cannot already access.
- Implementing rate limiting (orthogonal feature).