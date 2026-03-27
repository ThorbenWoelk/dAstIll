# Spec: Channel Navigation And Audio Resilience

## Goal

Fix workspace and channel-overview navigation so transient UI state remains coherent while users move between videos and channel pages, and fix the summary audio timeline so playback progress reflects the real audio length.

## Problems

- Generating summary audio is tracked only inside the mounted audio player component, so switching videos clears the in-flight state and forces the user to rediscover whether audio is still generating.
- Navigating from a video in one channel to another channel overview recreates the per-channel preview sidebar in a state that expands the wrong channel and can visually focus stale context.
- The channel overview route reloads the full channel list on every selected-channel change instead of reusing the already loaded list when possible.
- The summary audio timeline can clamp at the end when duration metadata is not known yet, so the thumb appears stuck at 100%.

## Requirements

- Audio summary generation/loading state must survive changing selected videos and survive returning to a video later in the same session.
- Newly introduced UI state should be resilient to navigating back and forth.
- Per-channel preview navigation must prefer the current route/channel selection over stale or default expansion choices.
- Changing from one channel/video context to another channel overview must not unnecessarily reload the channel list.
- The audio timeline must not pin itself to the end while metadata is still resolving and should adopt the real duration as soon as it becomes available.

## Implementation Notes

- Persist audio player session state in a shared frontend store/helper keyed by `videoId`.
- Extract sidebar expansion selection rules into a testable helper and use it for initial expansion in per-channel preview mode.
- Split channel overview loading into channel-list loading and selected-channel-depth loading so route changes can reuse existing list data.
- Prefer pure helpers for new decision logic so regression tests stay cheap and targeted.
