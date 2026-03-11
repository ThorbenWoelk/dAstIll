# Tasks: Video Info Duration Repair

## Current State

Implemented the stale-cache refresh path and extended the YouTube watch-page parser to read duration from `ytInitialPlayerResponse`. Full verification passed, and a live API smoke now returns duration for videos where YouTube exposes `lengthSeconds`.

## Steps

- [x] Add a failing test for the cached-video-info refresh decision.
- [x] Refresh incomplete cached video info when duration is missing.
- [x] Hide the duration detail in the frontend when duration is still unavailable.
- [x] Run frontend/backend verification for the touched paths.

## Decisions Made During Implementation

- Cached `video_info` is treated as incomplete only when both duration fields are missing or blank; other cached metadata still short-circuits the handler.
- If a refresh attempt fails for an incomplete cached row, the backend returns the cached metadata instead of dropping to the minimal fallback payload.
- The watch-page parser now supplements JSON-LD with `ytInitialPlayerResponse`, which is where current YouTube pages expose `lengthSeconds` for at least some videos.
