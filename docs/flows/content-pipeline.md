# Content Pipeline

## End-to-End View

```text
Channel input
  -> channel resolution
  -> video discovery
  -> video queue state
  -> transcript extraction
  -> summary generation
  -> summary evaluation
  -> search source sync
  -> search chunk indexing (S3 + S3 Vectors)
  -> retrieval in workspace search
```

User interactions:

```text
Highlight creation -> stored in highlights table -> grouped in /highlights route
Acknowledgement -> stored on video record -> filters inbox view
```

## 1. Channel Subscription

When a user adds a channel:

1. the backend resolves the input to a canonical channel id
2. channel metadata is stored
3. an async task fetches current videos for initial population

This is a write to canonical state first. It does not wait for transcript or summary generation.

## 2. Video Discovery

Videos enter the system from multiple paths:

- initial subscription sync
- periodic refresh worker
- historical gap scan worker
- explicit channel backfill

Inserted videos begin with transcript and summary lifecycle states that the queue worker consumes.

## 3. Transcript Extraction

The queue worker processes transcripts before summaries whenever a video is missing a ready transcript.

Transcript extraction starts with the external `summarize` CLI to extract plain transcript text (and a formatted transcript representation).

When `summarize` returns empty output (or a placeholder blurb), the backend falls back to `yt-dlp` using the `json3` subtitle format to extract timed caption events.

Those timed events are parsed into `TimedSegment[]` and later stored as optional `start_sec` on transcript chunks for timestamp-aware search metadata.

On success:

- transcript rows are stored
- `videos.transcript_status` becomes `ready`
- the transcript search source is marked pending

On rate limit:

- transcript cooldown is activated
- video status is moved back to `pending`

## 4. Summary Generation

A summary is generated only after a ready transcript exists.

The backend:

1. loads transcript text
2. calls the summarizer model
3. stores the summary
4. marks `summary_status = ready`
5. marks the summary search source pending

Manual summary edits use the same canonical-save-then-search-sync pattern.

## 5. Summary Evaluation

The summary evaluation worker scans summaries with missing quality state.

It:

- compares transcript and summary content
- assigns a `quality_score`
- writes `quality_note` and `quality_model_used`

Low-scoring summaries can be requeued by setting the video summary state back to `pending`, subject to the configured regeneration attempt cap.

## 6. Search Synchronization Hook

Transcript and summary write paths do not embed or chunk content inline.

Instead they:

- compute a content hash
- mark the corresponding `search_sources` row `pending`

If content is removed or empty, the search source is cleared.

## 7. Search Indexing

The search worker later:

- discovers missing sources
- claims pending rows
- loads canonical content
- chunks it
- optionally embeds it
- writes derived chunk rows

This is what keeps write latency separated from retrieval maintenance.

## Failure Boundaries

The system is designed so that:

- canonical content can succeed even if search is offline
- transcript extraction can fail without corrupting videos
- summary evaluation can pause without blocking search
- local and cloud model issues degrade state rather than crash the app
