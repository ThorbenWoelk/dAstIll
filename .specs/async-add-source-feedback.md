# Async Add Source Feedback

## Problem

Adding a video URL or channel currently behaves as a synchronous navigation flow. The UI either jumps immediately into content that is still generating or gives too little confirmation that the submitted URL/channel was valid and is being processed.

## Goal

Make source additions feel explicitly asynchronous. After a user submits a valid video URL or channel, the app should show a subtle popup confirming the source was accepted and is loading, then prompt again when the source is ready with an explicit option to jump to it.

## Requirements

- Adding a valid YouTube video URL must confirm acceptance without immediately navigating away.
- Adding a valid channel input must confirm acceptance without immediately navigating away.
- Invalid inputs must still surface the existing error path and keep the input available for correction.
- The popup copy must distinguish between "accepted/loading" and "ready to open" states.
- When a newly added video becomes ready, the popup must offer an explicit action to open that video.
- When a newly added channel has loaded enough to browse, the popup must offer an explicit action to open that channel.
- The channel overview route must support video URL input in the shared add form instead of treating every input as a channel subscription.
- The feedback flow should be shared across the main workspace and channel overview so behavior stays consistent.

## Non-Goals

- Building a multi-item notification center or persistent inbox.
- Changing backend ingestion semantics or queueing architecture.
- Adding browser push notifications or OS-level notifications.

## Design Considerations

- Keep the interaction lightweight: a toast-style popup is enough if it can update in place from loading to ready.
- Poll readiness from existing read APIs instead of introducing new backend endpoints unless frontend-only detection proves insufficient.
- Use pure helper functions for readiness/copy decisions so regression tests stay cheap.

## Open Questions

- Channel readiness is inferred from existing list/snapshot data and will be treated as "ready to open" once videos are available to browse.
