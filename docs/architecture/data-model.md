# Data Model

<script setup>
const storageOwnershipDiagram = String.raw`
flowchart LR
  subgraph s3canonical["S3-backed canonical records"]
    channels[channels]
    transcripts[transcripts]
    summaries[summaries]
    videoinfo[video_info]
  end

  subgraph firestorecanonical["Firestore-backed records"]
    videos[videos]
  end

  subgraph userstate["User-scoped S3 records"]
    subscriptions[user-channel-subscriptions]
    memberships[user-video-memberships]
    videostate[user-video-states]
    highlights[user-highlights]
    chats[user-conversations]
  end

  subgraph search["Derived search projection"]
    sources[search_sources]
    chunks[search_chunks]
    vectors[S3 Vectors embeddings]
    fts[libSQL BM25 / FTS5]
  end

  subgraph firestore["Firestore"]
    prefs[dastill_preferences]
    tts[dastill_tts_stats]
  end

  channels --> videos
  videos --> transcripts
  videos --> summaries
  videos --> videoinfo
  transcripts --> sources
  summaries --> sources
  sources --> chunks
  chunks --> vectors
  chunks --> fts
  subscriptions --> videos
  memberships --> videos
  videostate --> videos
`;

const searchProjectionDiagram = String.raw`
flowchart LR
  transcript[Transcript content]
  summary[Summary content]
  pending[Mark search_sources pending]
  worker[Search index worker]
  chunks[search_chunks objects]
  fts[libSQL / Turso FTS5]
  vectors[S3 Vectors]
  results[Search + chat retrieval]

  transcript --> pending
  summary --> pending
  pending --> worker
  worker --> chunks
  worker --> fts
  worker --> vectors
  fts --> results
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
- `Video` responses combine the canonical Firestore-backed `videos` record with the caller's
  `user-video-states/{user_id}/{video_id}.json` overlay, which currently carries
  `acknowledged`.
- Anonymous requests do not persist these user-scoped records. They operate against the
  seeded default channel scope exposed by `AccessContext`.

## Search Projection

Search is intentionally modeled as a derived projection stored in S3:

| Storage          | Role                                           |
| ---------------- | ---------------------------------------------- |
| `search_sources` | Per-video, per-source indexing lifecycle state |
| `search_chunks`  | Chunked search content stored as S3 objects    |
| S3 Vectors Index | Vector embeddings for semantic search          |

S3 Vectors provides managed ANN vector storage and retrieval for semantic search.

The backend also maintains a **libSQL/Turso BM25 keyword index**. In production it runs
through a Turso primary plus a local embedded replica file; locally it can fall back to a
plain libSQL file. All keyword search queries go through this index - there is no
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

## Firestore Collections

The application uses Google Firestore for video records plus selected user-facing state and statistics.

### Video Records (`dastill_videos`)

Canonical video records, queue state, retry counts, and summary quality mirrors are stored in
Firestore rather than S3.

| Field group                       | Description                                           |
| --------------------------------- | ----------------------------------------------------- |
| metadata                          | Video id, channel id, title, publish timestamp        |
| processing state                  | `transcript_status`, `summary_status`, `retry_count`  |
| quality mirror                    | `quality_score` copied from summary evaluation output |
| user-facing flags in API overlays | `acknowledged` merged at read time from user state    |

### User Preferences (`dastill_preferences`)

Per-authenticated-user preferences stored in Firestore:

| Field                     | Description                            |
| ------------------------- | -------------------------------------- |
| `channel_order`           | Ordered list of channel IDs            |
| `channel_sort_mode`       | Sort mode: `custom`, `alpha`, `newest` |
| `vocabulary_replacements` | Custom word replacements for summaries |

Document IDs use the authenticated Firebase user id. On first authenticated access, the
backend can copy a legacy `dastill_preferences/user` document forward for compatibility.

### TTS Statistics (`dastill_tts_stats`)

Aggregated text-to-speech generation metrics stored in the global Firestore document
`dastill_tts_stats/global`:

| Field                 | Description                              |
| --------------------- | ---------------------------------------- |
| `sample_count`        | Number of completed TTS generations      |
| `total_words`         | Cumulative words processed               |
| `total_duration_secs` | Cumulative synthesis duration in seconds |

Used to estimate synthesis time for new TTS requests.

---

## Storage Ownership Summary

| Data                                  | Storage    | Notes                                                     |
| ------------------------------------- | ---------- | --------------------------------------------------------- |
| Channels                              | S3         | `channels/{id}.json` canonical channel records            |
| Videos                                | Firestore  | `dastill_videos` canonical video records and queue status |
| Transcripts, summaries, video info    | S3         | Canonical content blobs                                   |
| User channel subscriptions            | S3         | `user-channel-subscriptions/{user_id}`                    |
| User video memberships and view state | S3         | `user-video-memberships/*` and `user-video-states/*`      |
| Search chunks                         | S3         | Derived projection                                        |
| Search sources                        | S3         | Derived projection metadata                               |
| Vector embeddings                     | S3 Vectors | Semantic search                                           |
| Conversations                         | S3         | Authenticated user chat history                           |
| Highlights                            | S3         | Authenticated user annotations                            |
| User preferences                      | Firestore  | Per-user settings                                         |
| TTS statistics                        | Firestore  | Global synthesis metrics                                  |

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
