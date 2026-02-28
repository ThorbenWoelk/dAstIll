# Spec: Channel List Loading Must Not Block on Background Refresh

## Problem
On both workspace and download queue pages, the Channels panel can remain in the loading skeleton state for a long time (or appear stuck) during initial page load.

## Root Cause
`loadChannels()` awaits `selectChannel()` when a first channel exists. `selectChannel()` awaits `refreshAndLoadVideos()`, which includes a background refresh API call. If that refresh is slow/failing, channel-list loading state remains true.

## Requirements
- Channel list loading state must only represent channel list fetch, not downstream video refresh work.
- Initial channel auto-selection should still happen.
- Existing video refresh behavior should remain unchanged.

## Acceptance Criteria
- Channels skeleton disappears after `listChannels()` resolves.
- A selected channel still auto-loads videos.
- Long-running refresh no longer keeps Channels panel in perpetual refresh UI.
