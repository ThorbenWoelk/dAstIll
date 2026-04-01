# Tasks: Transcript Extraction Robustness

## Current State

The transcript fallback now prefers `yt-dlp -J` caption metadata plus direct `json3` fetches and keeps the legacy subtitle-file path only as backup. Targeted backend transcript tests are passing. Investigation also confirmed that `summarize` can return a cached HTML extraction for these videos when its earlier transcript lookup was cached as unavailable, which is what produced the short `Sup nerds...` snippet.
Production storage scan found three cached poisoned transcript objects containing the `Sup nerds we got things to discuss.` blurb: `nDU7Mn-XRWI`, `nsqGI1VAYbU`, and `5qA0HessLi4`. The cleanup is complete: no transcript object in S3 contains that snippet anymore, `nsqGI1VAYbU` and `5qA0HessLi4` are reset to missing transcript/summary artifacts with Firestore statuses at `pending`, and `nDU7Mn-XRWI` regenerated a clean transcript while its summary remains `pending`.

## Steps

- [x] Add regression tests for current `yt-dlp` fallback behavior and truncated summarize output.
- [x] Patch the transcript fallback path to consume current `yt-dlp` caption metadata robustly.
- [x] Verify the summarize root cause and record the findings in code comments or task notes.
- [x] Run targeted backend verification for transcript extraction behavior.

## Decisions Made During Implementation

- Prefer using `yt-dlp -J` caption metadata plus direct `json3` fetches over filesystem subtitle outputs.
- Keep the old file-based subtitle lookup as a backup path in case `yt-dlp` metadata does not expose a usable caption URL.
- Treat the bad `summarize` output as a cached HTML extraction problem, not a bad caption-track problem, because fresh-cache runs resolve the full `captionTracks` transcript for the same videos.
