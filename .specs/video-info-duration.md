# Video Info Duration Repair

**Linear:** none

## Problem

The workspace info view often shows `Duration Unknown` even though the backend data model and YouTube parser support duration fields. This makes the info panel look broken and keeps stale incomplete metadata around indefinitely.

## Goal

Video duration should display whenever it can be derived from YouTube metadata. Cached incomplete video info should refresh instead of permanently serving missing duration values. If duration still cannot be obtained after refresh, the UI should avoid presenting misleading placeholder detail.

## Requirements

- Cached `video_info` rows with missing duration must not block a refresh attempt.
- Freshly fetched video info with duration must be persisted and returned.
- The frontend info panel must not render a `Duration Unknown` placeholder when duration is unavailable after refresh.
- Automated tests must cover the new cache-refresh decision logic.

## Non-Goals

- No schema migration for existing `video_info` rows.
- No broader redesign of the info panel.
- No changes to unrelated metadata fields beyond what is needed for duration handling.

## Design Considerations

- Prefer refreshing only obviously incomplete cached rows to avoid unnecessary repeated upstream requests.
- Keep the UI conservative: hide duration detail entirely when the backend still cannot resolve it.

## Open Questions

- None.
