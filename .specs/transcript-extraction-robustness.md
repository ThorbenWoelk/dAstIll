# Transcript Extraction Robustness

## Problem

Recent T3 videos were stored with truncated transcripts that contain only the first cue from `summarize`, even though full automatic captions are available for the same videos. The current fallback path also depends on `yt-dlp` writing a local `json3` subtitle file, which is brittle against current `yt-dlp` behavior.

## Goal

Transcript extraction should reliably recover full caption text when `summarize` returns a truncated snippet, and the codebase should document or expose the root cause behind the incorrect `summarize` transcript behavior.

## Requirements

- The transcript fallback path must remain compatible with the current production `yt-dlp` version.
- When `summarize` returns a short first-cue snippet, the backend must be able to recover full timed captions from `yt-dlp`.
- The fallback path must preserve timed segments when captions are available.
- The backend must have regression coverage for the current `yt-dlp` fallback behavior.
- The implementation must leave a clear explanation of why `summarize` produced incorrect transcripts for the affected videos.

## Non-Goals

- Replacing `summarize` as the primary transcript extractor.
- Reprocessing or repairing existing production transcript records in this change.
- Changing unrelated summary or search indexing behavior.

## Design Considerations

- The most robust fallback is to consume the caption URL from `yt-dlp -J` metadata instead of relying on subtitle files being written to disk.
- The `summarize` issue needs root-cause evidence from the live command behavior, not speculation.
- Tests should target parsing and fallback-selection behavior without depending on live network access.

## Open Questions

- Whether `summarize` is internally consuming only the first cue for some livestream/VOD variants or failing against a specific caption track.
