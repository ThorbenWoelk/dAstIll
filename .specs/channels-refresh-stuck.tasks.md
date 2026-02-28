# Tasks: Channel List Loading Must Not Block on Background Refresh

## Current State
Fix implemented and verified: channel list loading now clears after `listChannels()` resolves, while initial channel selection proceeds afterward.

## Steps
- [x] Identify where Channels loading state is toggled
- [x] Confirm coupling with select/refresh flow
- [x] Decouple channel list loading from channel refresh flow
- [x] Run frontend checks/build
- [x] Document results and residual risks

## Decisions Made During Implementation
- Keep initial channel auto-selection behavior, but perform it after channel list loading is cleared.
- Guard delayed auto-selection with `!selectedChannelId` to avoid replacing a user selection during async flow.
- Apply the same fix to both workspace and download queue pages for consistent behavior.
