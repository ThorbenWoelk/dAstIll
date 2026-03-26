# Tasks: Summary Audio Pivot

## Current State
Implementation complete. Backend supports generation via `POST` and retrieval from cache via `GET`. Frontend provides a new multi-state player.

## Steps
- [x] Backend: Modify `content.rs` to handle `POST` for generation and update `GET` for cache-only.
- [x] Backend: Update `main.rs` to register the new `POST` route.
- [x] Frontend: Update `WorkspaceSummaryAudioPlayer.svelte` with the new UI state machine.
- [x] Frontend: Test the full flow.

## Decisions Made During Implementation
- `GET /api/videos/{id}/summary/audio` will now only return cached audio.
- A new `POST /api/videos/{id}/summary/audio` will trigger TTS generation and cache the result.
- Cache key is video ID + content hash + TTS config hash.
