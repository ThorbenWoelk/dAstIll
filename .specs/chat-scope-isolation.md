# Chat Scope Isolation

## Problem

The active chat runtime registry is effectively global by conversation id, which allows operations like delete-all, reconnect, and cancel to operate without strict caller scope isolation. In-flight chat ownership is therefore weaker than the persisted conversation scope model and can allow one user to interfere with another user's active chat state.

## Goal

Make active chat runtime ownership strictly scope-aware so only the owning user or anonymous scope can reconnect to, cancel, or bulk-cancel its own in-flight chat work.

## Requirements

- Scope active in-memory chat handles by both scope id and conversation id.
- Ensure delete-all only cancels active chats inside the caller's scope.
- Ensure reconnect and cancel only operate on an active chat owned by the caller's scope.
- Keep anonymous ephemeral chat state isolated from authenticated conversation scope.
- Add regression coverage for cross-user isolation and id collision scenarios.

## Non-Goals

- Changing the existing chat HTTP routes or payload shapes.
- Redesigning the persisted conversation storage model.
- Reworking unrelated chat planner or synthesis behavior outside ownership checks.

## Design Considerations

- `AccessContext` already defines the correct ownership boundary, so runtime state should derive from the same scope key used for persisted conversation access.
- Returning `404` for foreign conversations avoids leaking the existence of another user's active conversation.
- Anonymous and authenticated chat ids must not collide in the runtime registry even if the raw conversation ids match.

## Open Questions

- None at the moment. The ownership boundary and desired API behavior are clear.
