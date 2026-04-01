# Chat Runtime Scope Isolation

**Linear:** n/a

## Problem

The active in-memory chat registry is keyed only by conversation id. That lets runtime operations such as reconnect, cancel, and delete-all act on whichever in-flight chat happens to share that id, even when the caller belongs to a different ownership scope.

## Goal

Active chat runtime state is isolated by the same ownership boundary as persisted conversation access, so only the owning authenticated user or the anonymous scope can reconnect to, cancel, or bulk-cancel its own in-flight chat work.

## Requirements

- Scope active in-memory chat handles by both ownership scope and conversation id.
- Delete-all only cancels active chats that belong to the caller's scope.
- Reconnect and cancel only operate on an active chat owned by the caller's scope and return `404` for foreign active conversations.
- Anonymous runtime chat state remains isolated from authenticated user scope even when raw conversation ids match.
- Add regression coverage for cross-user isolation and runtime id collision scenarios.

## Non-Goals

- Changing the existing chat HTTP routes or payload shapes.
- Redesigning persisted conversation storage.
- Refactoring unrelated planner, retrieval, or synthesis behavior outside ownership checks.

## Design Considerations

- `AccessContext` already defines the ownership boundary, so runtime active-chat keys should derive from that scope identity instead of raw conversation ids alone.
- Returning `404` for foreign active conversations preserves the existing API shape and avoids leaking whether another scope has in-flight work.
- Runtime keys should distinguish anonymous and authenticated scopes even when a raw conversation id is reused.

## Open Questions

- None at the moment. The ownership boundary and API behavior are already defined.
