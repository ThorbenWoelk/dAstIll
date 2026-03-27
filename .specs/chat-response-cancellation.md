# Chat Response Cancellation

## Problem

The chat backend exposes a cancel endpoint, but cancellation is only checked inside the final token streaming loop. If a user cancels while the service is still planning, retrieving sources, running tool calls, or synthesizing evidence, the request can continue running until those stages finish. In practice this makes the cancel control feel broken for slower responses.

## Goal

Cancelling an in-progress chat response should stop the backend pipeline promptly across all major pre-generation stages and persist a cancelled assistant message instead of continuing work until a later phase.

## Requirements

- Cancelling an active chat response must be honored before final token streaming starts.
- The chat service must stop work during long-running planning, retrieval, and synthesis-preparation phases when cancellation is requested.
- A cancelled response must persist as a cancelled assistant message rather than a rejected or completed response.
- Add regression coverage for the cancellation behavior that previously failed.

## Non-Goals

- Redesigning the chat UI or changing the cancel button interaction model.
- Refactoring the full chat pipeline beyond what is needed to make cancellation reliable.
- Changing unrelated active chat or conversation persistence behavior.

## Design Considerations

- The existing `ActiveChatHandle` already owns the cancellation signal, so the smallest safe change is to expose lightweight cancellation checks and use them at each async boundary.
- For long async operations such as planner/model calls, cancellation should race the in-flight future rather than waiting for the future to finish.
- Local search and data operations are shorter-lived, so explicit boundary checks are sufficient there.

## Open Questions

- None at the moment. The expected behavior is clear from the existing API contract and UI control.
