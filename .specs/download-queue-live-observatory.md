# Download Queue Live Observatory

## Problem
Users can see queued videos but cannot easily tell if queue processing is actively progressing without repeatedly refreshing the page manually.

## Goal
Make queue progress inspection obvious in the Download Queue UI through live refresh controls and clear progress telemetry.

## Requirements
- Add optional auto-refresh for the selected channel queue.
- Show last successful sync time in the queue header.
- Show a compact progress summary that makes movement obvious between syncs.
- Keep existing queue list behavior and channel selection intact.

## Non-Goals
- New backend APIs or schema changes.
- Queue retry controls.
- Visual redesign outside the Download Queue page.

## Design Considerations
Use existing `listVideos` endpoint polling with a conservative interval to avoid extra backend load. Keep controls near queue stats for immediate visibility.

## Open Questions
- None for current scope.
