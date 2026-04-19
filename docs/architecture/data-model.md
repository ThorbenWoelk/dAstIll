---
aside: false
---

# Data Model

<script setup>
const storageOwnershipDiagram = String.raw`
flowchart TB
  canonical[Canonical content]
  audio[Generated audio cache]
  userstate[User-scoped state]
  search[Derived search]
  appstate[local libSQL app state]

  canonical --> audio
  canonical --> search
  userstate --> canonical
`;

const searchProjectionDiagram = String.raw`
flowchart TB
  transcript[Transcript content]
  summary[Summary content]
  pending[Mark search_sources pending]
  worker[Search index worker]
  chunks[search_chunks]
  keyword[Keyword index<br/>local libSQL]
  vectors[Semantic index<br/>S3 Vectors]
  results[Search + chat]

  transcript --> pending
  summary --> pending
  pending --> worker
  worker --> chunks
  worker --> keyword
  worker --> vectors
  keyword --> results
  vectors --> results
`;
</script>

## Canonical Record Sets

These are the authoritative content records owned by ingest and processing, not user
overlays or derived search projections.

| Table         | Role                                                                  |
| ------------- | --------------------------------------------------------------------- |
| `channels`    | Canonical channel metadata discovered from YouTube                    |
| `videos`      | Canonical per-video metadata plus transcript/summary processing state |
| `transcripts` | Extracted raw text and formatted markdown transcript forms            |
| `summaries`   | Generated or manually edited summaries plus quality fields            |
| `video_info`  | Extended metadata such as description, duration, and view count       |

<MermaidDiagram
  caption="Storage ownership map: canonical content, user-scoped records, and derived search state are separated so indexing can be rebuilt without rewriting source records."
  :chart="storageOwnershipDiagram"
/>

## Core Status Fields

`videos` carries two key lifecycle fields:

- `transcript_status`
- `summary_status`

Each can be:

- `pending`
- `loading`
- `ready`
- `failed`

These statuses drive the queue worker and much of the UI state.

Additional video fields:

- `acknowledged` - user-scoped read state overlaid onto API responses
- `retry_count` - caps regeneration attempts for summaries
- `quality_score` - 0-10 rating from the evaluator model

## API View Models vs Stored Records

Some API payloads intentionally merge canonical and user-scoped records:

- `Channel` responses combine canonical channel metadata with the caller's
  `user-channel-subscriptions/{user_id}/{channel_id}.json` record.
- `Video` responses combine the canonical `libSQL`-backed `videos` row with the caller's
  `user-video-states/{user_id}/{video_id}.json` overlay, which currently carries
  `acknowledged`.
- Anonymous requests do not persist these user-scoped records. They operate against the
  seeded default channel scope exposed by `AccessContext`.

## Client-Side Storage

Browser storage is only a cache or UI-state layer. It is not the source of truth for
channels, videos, transcripts, summaries, or other canonical content.

- Canonical and user-owned records stay on the backend in the stores described above.
- IndexedDB, `localStorage`, and `sessionStorage` hold derived startup caches, layout
  preferences, and ephemeral draft state only.
- Any browser-stored user data must be keyed by auth scope so one signed-in user,
  signed-out visitor, or anonymous Firebase identity cannot read another scope's data
  from the same browser profile.

## Search Projection

Search is intentionally modeled as a derived projection stored in S3:

| Storage          | Role                                           |
| ---------------- | ---------------------------------------------- |
| `search_sources` | Per-video, per-source indexing lifecycle state |
| `search_chunks`  | Chunked search content stored as S3 objects    |
| S3 Vectors Index | Vector embeddings for semantic search          |

S3 Vectors provides managed ANN vector storage and retrieval for semantic search.

The backend also maintains a **libSQL BM25 keyword index**. Each backend instance keeps a
local libSQL file and rebuilds the keyword index from stored search artifacts when needed. All keyword
search queries go through this index - there is no
per-query S3 scan. The keyword index is kept in sync by the search index worker after
every write and can be rebuilt from the stored `search-chunks/` projection when empty.

<MermaidDiagram
  caption="Canonical transcript and summary records feed the derived search projection, which then powers both keyword and semantic retrieval."
  :chart="searchProjectionDiagram"
/>

### `search_sources`

Tracks one record per `(video_id, source_kind)` pair with:

- `content_hash`
- `source_generation`
- `embedding_model` - stores the configured embedding model string used for that generation
- `index_status`
- `last_indexed_at`
- `last_error`

### `search_chunks`

Each chunk is stored as an S3 object with:

- `search_source_id`
- `source_generation`
- `chunk_index`
- `section_title`
- `chunk_text`
- `start_sec` (optional) - start position in the video (seconds) for timestamp-aware transcript chunks
- `token_count`

Embeddings are stored separately in S3 Vectors.

## Generated Audio Cache

Summary audio is not treated as canonical source content. It is a derived cache generated from the current summary text when Polly TTS is enabled.

- storage key shape: `summary-audio/{video_id}/{audio_hash}.{ext}`
- cache invalidation key: a hash of the current summary content plus the active Polly voice/engine/output settings
- read path: `GET /api/videos/{id}/summary/audio`
- generation path: `POST /api/videos/{id}/summary/audio`

If the summary changes or the TTS settings change, the cache key changes and the old audio is no longer reused.

## User-Scoped Library Records

Most user-owned library state lives in S3 under user-specific prefixes.

### Subscriptions and per-user video state

| Prefix                                  | Role                                                       |
| --------------------------------------- | ---------------------------------------------------------- |
| `user-channel-subscriptions/{user_id}/` | Channel subscriptions plus per-subscription sync settings  |
| `user-video-memberships/{user_id}/`     | Explicitly added videos, including the virtual Others view |
| `user-video-states/{user_id}/`          | Per-user overlays such as `acknowledged`                   |

These records are loaded into `AccessContext` at request time so the backend can scope
channel, video, search, and chat access to the caller's library.

### Highlights

Highlights are stored per authenticated user under `user-highlights/{user_id}/`.

Each highlight stores:

- `id` - unique identifier
- `video_id` - associated video
- `source` - `transcript` or `summary`
- `text` - the highlighted content
- `prefix_context` / `suffix_context` - surrounding text for context
- `created_at` - timestamp

The `/highlights` route groups these per-user records by channel and video at read time.

### Chat Storage

Persistent chat conversations are stored in S3 as JSON objects under authenticated user
scope, separate from canonical content:

| Storage                                   | Role                                               |
| ----------------------------------------- | -------------------------------------------------- |
| `user-conversations/{user_id}/index.json` | Conversation list index with titles and timestamps |
| `user-conversations/{user_id}/{id}.json`  | Full conversation with all messages and sources    |

### Conversation Structure

Each conversation contains:

- `id` - unique identifier
- `title` - auto-generated or user-set title
- `created_at` / `updated_at` - timestamps
- `messages` - ordered list of messages

### Message Structure

Each message includes:

- `id` - unique identifier
- `role` - `user` or `assistant`
- `content` - the message text
- `status` - `pending`, `streaming`, `complete`, or `failed`
- `sources` - retrieved chunks used for RAG grounding (assistant messages only)

Sources reference the search index and provide attribution for AI responses.

### Why Separate Chat Storage

Chat is intentionally separate from canonical content:

- conversations are ephemeral user interactions, not canonical content
- messages reference existing search chunks but don't duplicate them
- conversations can be deleted without affecting transcripts or summaries
- signed-out chat stays in the frontend's ephemeral session path instead of writing these
  S3 objects

---

## libSQL Tables

The application uses a shared SQL-backed storage layer for canonical video records plus selected
user-facing state and statistics. The runtime stores those tables in a local `libSQL` file and
reconciles from S3-backed snapshots when the local cache is empty.

### Video Records (`videos`)

Canonical video records, queue state, retry counts, and summary quality mirrors are stored in the
`videos` table rather than S3.

| Field group                       | Description                                           |
| --------------------------------- | ----------------------------------------------------- |
| metadata                          | Video id, channel id, title, publish timestamp        |
| processing state                  | `transcript_status`, `summary_status`, `retry_count`  |
| quality mirror                    | `quality_score` copied from summary evaluation output |
| user-facing flags in API overlays | `acknowledged` merged at read time from user state    |

### User Preferences (`preferences`)

Per-authenticated-user preferences stored in the `preferences` table:

| Field                     | Description                            |
| ------------------------- | -------------------------------------- |
| `channel_order`           | Ordered list of channel IDs            |
| `channel_sort_mode`       | Sort mode: `custom`, `alpha`, `newest` |
| `vocabulary_replacements` | Custom word replacements for summaries |

Rows use the authenticated Firebase user id as `user_id`. On first authenticated access, the
backend can copy a legacy single-user row with `preferences.user_id = "user"` forward for
compatibility.

### TTS Statistics (`tts_stats`)

Aggregated text-to-speech generation metrics stored in the global `tts_stats` row with
`id = "global"`:

| Field                 | Description                              |
| --------------------- | ---------------------------------------- |
| `sample_count`        | Number of completed TTS generations      |
| `total_words`         | Cumulative words processed               |
| `total_duration_secs` | Cumulative synthesis duration in seconds |

Used to estimate synthesis time for new TTS requests.

---

## Storage Ownership Summary

| Data                                  | Storage    | Notes                                                       |
| ------------------------------------- | ---------- | ----------------------------------------------------------- |
| Channels                              | S3         | `channels/{id}.json` canonical channel records              |
| Videos                                | libSQL     | `videos` table for canonical video records and queue status |
| Transcripts, summaries, video info    | S3         | Canonical content blobs                                     |
| User channel subscriptions            | S3         | `user-channel-subscriptions/{user_id}`                      |
| User video memberships and view state | S3         | `user-video-memberships/*` and `user-video-states/*`        |
| Search chunks                         | S3         | Derived projection                                          |
| Search sources                        | S3         | Derived projection metadata                                 |
| Vector embeddings                     | S3 Vectors | Semantic search                                             |
| Conversations                         | S3         | Authenticated user chat history                             |
| Highlights                            | S3         | Authenticated user annotations                              |
| User preferences                      | libSQL     | `preferences` table keyed by `user_id`                      |
| TTS statistics                        | libSQL     | `tts_stats` table global aggregate row                      |

---

## Why Separate Canonical and Search Tables

This lets the app:

- rebuild search without rewriting canonical content
- change chunking and indexing behavior independently
- keep user-facing writes fast
- isolate failures in embedding or search projection work

## Derived State Rules

### Canonical writes queue search work

Transcript and summary changes do not inline-rebuild embeddings. They mark the relevant search source pending.

### Search chunks are disposable

If the projection schema changes, the backend can drop and recreate `search_sources` and `search_chunks` while preserving canonical transcript and summary content. S3 Vectors embeddings can be rebuilt independently.

## Counting Search Coverage

Search coverage totals intentionally use readiness flags from `videos` rather than scanning large transcript or summary text tables. This keeps status payloads lightweight for the startup bootstrap request.
