# Spec: Summary Audio Pivot

## Problem
Currently, summary audio is synthesized on-the-fly for every request. This is inefficient and doesn't allow for a "generate once, cache forever" approach. The user wants to trigger audio generation manually and have it cached in S3.

## Goals
1. Provide a manual "Download" trigger for audio generation.
2. Cache generated audio in S3 permanently.
3. Update the UI to reflect the generation state (Download, Loading, Play, Pause).
4. Efficiently check if audio exists before trying to play it.

## Architecture
- **Backend:**
    - `POST /api/videos/{id}/summary/audio`: Generates audio, caches to S3, returns metadata.
    - `GET /api/videos/{id}/summary/audio`: Checks S3 cache first. If hit, returns audio. If miss, returns 404 (indicating it needs generation).
    - Cache key should be derived from video ID and a hash of the summary content to invalidate if the summary changes.
- **Frontend:**
    - `WorkspaceSummaryAudioPlayer.svelte`:
        - New button replacing the default `<audio controls>`.
        - States: `missing` (Download), `generating` (Spinner), `ready` (Play), `playing` (Pause).
        - Handle playback via a hidden `<audio>` element.

## Tasks
- [ ] Backend: Add `POST /api/videos/{id}/summary/audio` handler.
- [ ] Backend: Update `GET /api/videos/{id}/summary/audio` to use cache.
- [ ] Backend: Register routes in `main.rs`.
- [ ] Frontend: Update `WorkspaceSummaryAudioPlayer.svelte` with new button and states.
- [ ] Frontend: Add API call for triggering generation.
- [ ] Frontend: Add API call (or use `HEAD`/status) for checking audio existence.
- [ ] Verification: Test generation, caching, and playback.
