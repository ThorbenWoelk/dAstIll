# Chat Recent Activity

## Summary

Improve chat handling for channel and creator prompts that ask what someone is doing lately, recently, or these days. The assistant should interpret these as requests about the most recent content available in the user's library, not as unsupported real-time knowledge requests.

## Goals

- Resolve plain-text channel and video references when unambiguous.
- Add a dedicated recent-activity retrieval path that is metadata-first instead of chunk-search-first.
- Improve answer quality for channel recency prompts so the assistant summarizes recent library content instead of leading with refusal.
- Preserve explicit honesty for truly unsupported real-time questions.

## Scope

- Backend chat intent classification
- Backend mention and plain-entity scope resolution
- Backend chat tool loop and tool outputs
- Backend recent-activity retrieval
- Tests for intent, scope, tool parsing, and recent retrieval output

## Out of Scope

- Frontend UI changes
- Full support for recent activity around a single scoped video
- Whole-library "what's new this week" aggregation beyond the current scoped-channel implementation

## Functional Requirements

1. Prompts containing `recent`, `recently`, `lately`, `latest`, `these days`, `currently`, `right now`, or `nowadays` should default to library recency behavior unless they explicitly ask for off-library live status.
2. Plain-text exact matches for channel names, handles, and video titles should be turned into internal scope when unambiguous.
3. A dedicated `recent_library_activity` internal chat tool should return the latest processed videos for a scoped channel, including summary snippets and transcript fallback snippets.
4. Recent-activity prompts should use the new tool first and only fall back to `search_library` when needed.
5. Recent-activity answers should summarize themes across recent videos and should not lead with refusal when at least two recent processed videos are available.
6. Explicit real-time questions like "is X live right now?" should still say the library cannot answer real-time status.

## Acceptance Criteria

- `What is HealthyGamerGG doing lately?` produces a recent-content summary from the library.
- Tool traces for recent channel prompts show recent-activity retrieval, not a weak keyword search for `recent topics`.
- Plain-text `HealthyGamerGG` and `Theo` can resolve to known channels when exact matches exist.
- Existing `@` and `+` scoping remains intact.
- The backend passes targeted tests, full `cargo test`, and `cargo build --release`.
