---
aside: false
---

# Content Pipeline

<script setup>
const contentPipelineDiagram = String.raw`
flowchart TB
  channel[Channel add,<br/>refresh, or backfill]
  discover[Video discovery]
  queue[Queue worker]
  transcript[Transcript ready]
  podcast_asr[Local podcast ASR<br/>OpenAI-compatible STT service]
  summary[Summary ready]
  eval[Summary evaluation]
  searchpending[Mark search pending]
  searchworker[Search index worker]
  projection[Search projection]
  retrieval[Workspace search + chat]

  channel --> discover
  discover --> queue
  queue --> transcript
  queue --> podcast_asr --> transcript
  transcript --> summary
  summary --> eval
  transcript --> searchpending
  summary --> searchpending
  eval --> searchpending
  searchpending --> searchworker
  searchworker --> projection
  projection --> retrieval
`;

const userScopedWritesDiagram = String.raw`
flowchart TB
  ui[User actions in workspace]
  ack[Acknowledge video]
  hl[Create highlight]
  subscriptions[Update subscriptions]
  videostate[user-video-states]
  highlights[user-highlights]
  channelscope[user-channel-subscriptions]
  api[Backend read model]
  responses[Scoped responses]

  ui --> ack --> videostate
  ui --> hl --> highlights
  ui --> subscriptions --> channelscope
  channelscope --> api
  videostate --> api
  highlights --> api
  api --> responses
`;
</script>

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
Highlight creation -> stored in user-highlights/{user_id}/ -> grouped in /highlights route
Acknowledgement -> stored in user-video-states/{user_id}/{video_id}.json -> overlaid onto video responses
```

<MermaidDiagram
  caption="Primary content pipeline: discovery and queueing feed transcript and summary generation, then evaluation and search projection maintenance run asynchronously."
  :chart="contentPipelineDiagram"
/>

<MermaidDiagram
  caption="User-scoped writes stay separate from canonical ingest: overlays and highlights are stored under per-user prefixes and merged back into read models later."
  :chart="userScopedWritesDiagram"
/>

## 1. Channel Subscription

When a user adds a channel:

1. the backend resolves the input to a canonical channel id
2. canonical channel metadata is stored if missing
3. the authenticated user's subscription record is stored
4. an async task fetches current videos for initial population

This keeps canonical channel data and user-scoped library membership separate. It does
not wait for transcript or summary generation.

## 2. Video Discovery

Videos enter the system from multiple paths:

- initial subscription sync
- periodic refresh worker
- historical gap scan worker
- explicit channel backfill
- manual single-video adds

YouTube live streams are queued only after YouTube reports the broadcast as completed.
Upcoming or active streams are skipped because captions/transcripts are not stable until
the broadcast ends. Completed livestream transcripts are also treated as not ready for
a short grace period after `actualEndTime`. If extraction still returns short text that
mostly matches the YouTube description, the backend defers the transcript instead of
saving it and generating a summary from description copy.

Inserted videos begin with transcript and summary lifecycle states that the queue worker consumes.
User visibility to those videos is derived later from channel subscriptions and explicit
video memberships rather than from separate per-user video copies.

## 3. Transcript Extraction

The queue worker processes transcripts before summaries whenever a video is missing a ready transcript.

### YouTube videos

YouTube transcript extraction starts with the external `summarize` CLI to extract plain transcript text (and a formatted transcript representation).

When `summarize` returns empty output (or a placeholder blurb), the backend falls back to `yt-dlp` using the `json3` subtitle format to extract timed caption events.

Those timed events are parsed into `TimedSegment[]` and later stored as optional `start_sec` on transcript chunks for timestamp-aware search metadata.

### Podcast episodes

Podcast RSS show notes are not transcripts. During podcast feed sync, dAstIll stores:

- the episode metadata
- show notes as description/display content only
- publisher `podcast:transcript` references when present
- the RSS audio enclosure as `MediaAssetKind::SourceAudio`

If a publisher transcript reference exists, the backend fetches it through the same public-media URL policy used for audio. Supported publisher formats are plain text, VTT, SRT, Podcasting 2.0 JSON segments, and HTML.

When no publisher transcript exists, podcast transcription uses a separate operator-owned ASR service. The Rust backend is only the client. It downloads the public audio enclosure through a pinned, public-address-only media fetcher, sends the audio to `LOCAL_ASR_BASE_URL/v1/audio/transcriptions` as multipart form data, and stores the returned text as the canonical transcript.

The STT model runs outside the backend process. The recommended free local/prod model is NVIDIA Parakeet TDT 0.6B v3, served by a trusted OpenAI-compatible ASR implementation. Third-party wrapper repositories are implementation details and should not be treated as product dependencies unless they have enough maintenance signal for production.

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
