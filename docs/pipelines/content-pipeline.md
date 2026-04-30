# Content Pipeline

<script setup>
const contentPipelineDiagram = String.raw`
flowchart TB
  input[Source input]
  discovery[Discovery + sync]
  queue[Queue worker]
  transcript[Transcript ready]
  summary[Summary ready]
  eval[Evaluation]
  searchpending[Search source pending]
  searchworker[Search worker]
  retrieval[Workspace search + chat]

  input --> discovery
  discovery --> queue
  queue --> transcript
  transcript --> summary
  summary --> eval
  transcript --> searchpending
  summary --> searchpending
  eval --> searchpending
  searchpending --> searchworker
  searchworker --> retrieval
`;

const transcriptFlowDiagram = String.raw`
flowchart TB
  video[Queued video]
  youtube[YouTube transcript path]
  podcast[Podcast transcript path]
  publisher[Publisher transcript]
  asr[ASR service]
  stored[Canonical transcript]

  video --> youtube
  video --> podcast
  podcast --> publisher
  podcast --> asr
  youtube --> stored
  publisher --> stored
  asr --> stored
`;
</script>

## End-To-End Flow

<MermaidDiagram
  caption="Discovery and queueing feed transcript extraction, summary generation, evaluation, and search projection maintenance."
  :chart="contentPipelineDiagram"
/>

```text
source input
  -> source resolution
  -> video discovery
  -> queue state
  -> transcript extraction
  -> summary generation
  -> summary evaluation
  -> search source pending
  -> search indexing
  -> retrieval in workspace search and chat
```

## Source Subscription

When a user adds a source:

1. The backend resolves the input to a canonical source or channel id.
2. Canonical metadata is stored if missing.
3. The authenticated user's subscription or membership record is stored.
4. An async sync task fetches current videos or items.

The subscription path returns before transcript or summary generation completes.

Supported source-input forms for local smoke checks live in
[Local Development](/operations/local-development#smoke-test-inputs).

## Video Discovery

Videos enter the queue from:

- initial subscription sync
- periodic refresh worker
- historical gap scan worker
- explicit channel backfill
- manual single-video adds

Inserted videos start with transcript and summary lifecycle state for the queue worker.

YouTube livestream handling:

- upcoming and active streams are skipped
- completed livestreams wait for captions to stabilize after `actualEndTime`
- transcript output that still looks like description copy is deferred

User visibility is derived from subscriptions and explicit video memberships.

## Transcript Extraction

The queue worker processes transcripts before summaries whenever a video lacks a ready transcript.

<MermaidDiagram
  caption="Transcript extraction has separate YouTube and podcast paths. Both write canonical transcript records before summary generation."
  :chart="transcriptFlowDiagram"
/>

### YouTube

The YouTube path starts with the external `summarize` CLI.

If `summarize` returns empty output, a placeholder blurb, or text that appears truncated, the backend
falls back to `yt-dlp` with `json3` captions.

The `yt-dlp` fallback produces timed caption segments. Those timestamps later become `start_sec` on
transcript search chunks.

### Podcast

Podcast RSS show notes are stored as description/display content. They are not transcripts.

During podcast feed sync, dAstIll records:

- episode metadata
- show notes
- publisher `podcast:transcript` references when present
- RSS audio enclosure as source audio

Publisher transcript formats:

- plain text
- VTT
- SRT
- Podcasting 2.0 JSON segments
- HTML

If no publisher transcript exists, podcast transcription uses a separate ASR service. The Rust
backend is only the client.

ASR request shape depends on hosting:

- local or bearer-token endpoints receive downloaded audio bytes as multipart form data
- the repo-owned Cloud Run ASR service receives the validated public audio URL and fetches it
  server-side

Both paths store the returned `text` field or plain response body as the canonical transcript.

### Transcript Outcomes

On success:

- transcript content is stored
- `videos.transcript_status` becomes `ready`
- the transcript search source is marked pending

On rate limit or temporary transcript dependency failure:

- transcript cooldown is activated
- the video returns to `pending`

## Summary Generation

Summary generation runs only after a ready transcript exists.

The backend:

1. loads transcript text
2. calls the summarizer model
3. stores the summary
4. marks `summary_status = ready`
5. marks the summary search source pending

Manual summary edits use the same canonical-save-then-search-sync pattern.

## Summary Evaluation

The summary evaluation worker scans summaries with missing quality state.

It:

- compares transcript and summary content
- asks the evaluator for faithfulness and completeness scores
- requires defect evidence for non-perfect scored summaries
- writes quality score, quality note, model name, and tags
- can mark input as unscorable with a note and no numeric score

Low-scoring summaries can be requeued by setting summary state back to `pending`, subject to the
video retry cap.

## Search Sync

Transcript, summary, and evaluation writes do not chunk or embed content inline.

They:

- compute or update content state
- mark the corresponding search source pending
- let the search worker rebuild derived chunks and indexes later

This keeps write latency separate from retrieval maintenance.

## User-Scoped Writes

User interactions such as highlight creation, acknowledgement changes, and subscription updates
write user-scoped records. They do not rewrite canonical content.

## Failure Boundaries

The pipeline isolates failures by stage.

| Failure area          | Boundary                                                      |
| --------------------- | ------------------------------------------------------------- |
| Transcript extraction | video remains queued or pending; canonical video stays intact |
| Summary generation    | transcript stays ready; summary can retry later               |
| Summary evaluation    | summary remains usable; quality state can be filled later     |
| Search indexing       | canonical content remains written; retrieval catches up later |
| Model/cloud cooldown  | affected worker pauses; unrelated stages continue             |
