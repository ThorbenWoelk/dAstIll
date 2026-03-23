# Manual Video Ingest Into Others

## Goal

Allow users to paste an individual YouTube video without subscribing to the whole channel.

## Requirements

- Manually added videos from unsubscribed channels appear under a synthetic `Others` channel entry.
- If a manually added video belongs to a subscribed channel, it appears under that real channel instead of `Others`.
- If the video already exists in storage, do not duplicate or re-load it.
- Existing channel-centric flows keep working for subscribed channels.

## Approach

- Add a dedicated backend endpoint for manual video ingest.
- Expose a synthetic `Others` channel in channel-list/bootstrap/snapshot reads when orphan videos exist.
- Route the existing sidebar input to either channel subscription or single-video ingest based on the pasted input.
