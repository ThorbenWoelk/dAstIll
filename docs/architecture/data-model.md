# Data Model

<script setup>
const storageOwnershipDiagram = String.raw`
flowchart TB
  canonical[Canonical content]
  userstate[User-scoped state]
  search[Derived search]
  audio[Generated audio cache]
  sql[local libSQL tables]

  canonical --> search
  canonical --> audio
  userstate --> sql
  sql --> canonical
`;

const searchProjectionDiagram = String.raw`
flowchart TB
  transcript[Transcript content]
  summary[Summary content]
  sources[search_sources]
  chunks[search_chunks]
  vectors[S3 Vectors]
  keyword[local libSQL FTS]

  transcript --> sources
  summary --> sources
  sources --> chunks
  chunks --> vectors
  chunks --> keyword
`;
</script>

## Storage Ownership

<MermaidDiagram
  caption="Canonical content, user-scoped records, derived search state, generated audio, and local SQL tables have separate ownership boundaries."
  :chart="storageOwnershipDiagram"
/>

| Data                                  | Storage    | Notes                                                      |
| ------------------------------------- | ---------- | ---------------------------------------------------------- |
| Channels                              | S3         | `channels/{id}.json` canonical channel records             |
| Videos                                | libSQL     | `videos` table for canonical video records and queue state |
| Transcripts, summaries, video info    | S3         | Canonical content blobs                                    |
| User channel subscriptions            | S3         | `user-channel-subscriptions/{user_id}`                     |
| User video memberships and view state | S3         | `user-video-memberships/*` and `user-video-states/*`       |
| Search chunks                         | S3         | Derived projection                                         |
| Search sources                        | S3         | Derived projection metadata                                |
| Vector embeddings                     | S3 Vectors | Semantic search                                            |
| Conversations                         | S3         | Authenticated user chat history                            |
| Highlights                            | S3         | Authenticated user annotations                             |
| User preferences                      | libSQL     | `preferences` table keyed by `user_id`                     |
| TTS statistics                        | libSQL     | `tts_stats` table global aggregate row                     |

Browser storage is cache and UI state only. IndexedDB, `localStorage`, and `sessionStorage` do not
own canonical content or user records.

## Canonical Content

Canonical content is owned by ingest and processing. It is separate from user overlays and derived
search state.

| Record        | Role                                                                  |
| ------------- | --------------------------------------------------------------------- |
| `channels`    | Canonical channel metadata                                            |
| `videos`      | Canonical per-video metadata plus transcript/summary processing state |
| `transcripts` | Extracted raw text and formatted markdown transcript forms            |
| `summaries`   | Generated or manually edited summaries plus quality fields            |
| `video_info`  | Extended metadata such as description, duration, and view count       |

`videos` carries lifecycle state:

- `transcript_status`
- `summary_status`
- `retry_count`
- `quality_score`

The status values are:

- `pending`
- `loading`
- `ready`
- `failed`

User-facing fields such as `acknowledged` are overlays from user-scoped records.

## User-Scoped Library Records

Most user-owned library state lives in S3 under user-specific prefixes.

| Prefix                                  | Role                                                       |
| --------------------------------------- | ---------------------------------------------------------- |
| `user-channel-subscriptions/{user_id}/` | Channel subscriptions plus per-subscription sync settings  |
| `user-video-memberships/{user_id}/`     | Explicitly added videos, including the virtual Others view |
| `user-video-states/{user_id}/`          | Per-user overlays such as `acknowledged`                   |

These records are loaded into request scope so the backend can authorize and shape channel, video,
search, and chat responses for the caller.

## Highlights

Highlights are authenticated user annotations under `user-highlights/{user_id}/`.

Each highlight stores:

- `id`
- `video_id`
- `source`: `transcript` or `summary`
- `text`
- `prefix_context`
- `suffix_context`
- `created_at`

Grouping highlights by route or view is API behavior, not storage ownership.

## Chat Storage

Persistent chat conversations are authenticated user records in S3.

| Storage                                   | Role                                               |
| ----------------------------------------- | -------------------------------------------------- |
| `user-conversations/{user_id}/index.json` | Conversation list index with titles and timestamps |
| `user-conversations/{user_id}/{id}.json`  | Full conversation with messages and sources        |

Conversation records store:

- `id`
- `title`
- `created_at`
- `updated_at`
- `messages`

Message records store:

- `id`
- `role`
- `content`
- `status`
- `sources` for assistant messages

Sources reference search chunks for attribution. Signed-out chat stays in the frontend's ephemeral
session path and does not write these S3 records.

## Search Projection

Search is derived state. It can be rebuilt without rewriting canonical transcripts, summaries, or
video metadata.

<MermaidDiagram
  caption="Canonical transcript and summary records feed derived search source and chunk records. Search indexing then maintains keyword and vector indexes from those chunks."
  :chart="searchProjectionDiagram"
/>

### `search_sources`

One record per `(video_id, source_kind)` pair:

- `content_hash`
- `source_generation`
- `embedding_model`
- `index_status`
- `last_indexed_at`
- `last_error`

### `search_chunks`

Each chunk is an S3 object with:

- `search_source_id`
- `source_generation`
- `chunk_index`
- `section_title`
- `chunk_text`
- `start_sec`
- `token_count`

Embeddings are stored separately in S3 Vectors. The local libSQL FTS index is a runtime keyword
index built from `search_chunks`.

If the projection schema changes, the backend can drop and recreate `search_sources` and
`search_chunks`. S3 Vectors embeddings can be rebuilt independently.

Search coverage counts use readiness flags from `videos` instead of scanning large transcript or
summary text blobs.

## Generated Audio Cache

Summary audio is a derived cache generated from the current summary text when Polly TTS is enabled.

Storage key:

```text
summary-audio/{video_id}/{audio_hash}.{ext}
```

The cache key includes:

- current summary content
- Polly voice
- Polly engine
- Polly output settings

If the summary or TTS settings change, the cache key changes and old audio is not reused.

## libSQL Tables

The runtime stores selected app state in a local libSQL file and reconciles it from S3-backed
snapshots when needed.

### `videos`

Stores canonical video records, queue state, retry counts, and summary quality mirrors.

| Field group      | Description                                           |
| ---------------- | ----------------------------------------------------- |
| metadata         | Video id, channel id, title, publish timestamp        |
| processing state | `transcript_status`, `summary_status`, `retry_count`  |
| quality mirror   | `quality_score` copied from summary evaluation output |

### `preferences`

Per-authenticated-user preferences.

| Field                     | Description                            |
| ------------------------- | -------------------------------------- |
| `channel_order`           | Ordered list of channel IDs            |
| `channel_sort_mode`       | Sort mode: `custom`, `alpha`, `newest` |
| `vocabulary_replacements` | Custom word replacements for summaries |

Rows are keyed by Firebase user id. On first authenticated access, the backend can copy the legacy
single-user row `preferences.user_id = "user"` forward.

### `tts_stats`

Global TTS generation metrics in the row `id = "global"`.

| Field                 | Description                   |
| --------------------- | ----------------------------- |
| `sample_count`        | Completed TTS generations     |
| `total_words`         | Cumulative words processed    |
| `total_duration_secs` | Cumulative synthesis duration |

These values estimate synthesis time for future TTS requests.

## Storage Rules

### Canonical Before Derived

Canonical records are written first. Derived search state, generated audio, and runtime indexes can
be rebuilt later.

### User State Stays Scoped

User-owned records are keyed by authenticated user id or held in an ephemeral signed-out path.
Browser caches must be keyed by auth scope.

### Search Chunks Are Disposable

Search chunks and vector embeddings are derived from canonical transcript and summary content. They
can be dropped and rebuilt.
