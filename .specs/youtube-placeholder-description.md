# Spec: YouTube generic / placeholder video descriptions

## Symptom

Some videos show a long generic blurb, for example (German):

`Auf YouTube findest du die angesagtesten Videos und Tracks. Außerdem kannst du eigene Inhalte hochladen und mit Freunden oder gleich der ganzen Welt teilen`

## Cause

1. **Source**  
   The scraper reads `meta[property="og:description"]` and `meta[name="description"]` from the watch HTML. YouTube often injects **site-wide** marketing copy there when the page does not expose a video-specific snippet in meta tags (consent / layout / timing).

2. **Gap**  
   `ytInitialPlayerResponse.videoDetails.shortDescription` was not merged, so we never preferred the real per-video description when meta tags were generic.

## Mitigations

- Detect known **multi-locale** placeholder substrings and treat them as absent.
- Merge **`shortDescription`** from `ytInitialPlayerResponse` when meta / JSON-LD is missing or placeholder.
- **`cached_video_info_needs_refresh`**: re-fetch when stored description is a placeholder.
- **Backfill**: extend `POST /api/videos/info/backfill` with `heal_placeholders=true` to scan stored `video-info` JSON and re-fetch affected videos.

## Healing existing data

Operators run backfill with `heal_placeholders=true` (and appropriate `limit`) until `processed` reaches zero new fixes, or run repeatedly in batches.
