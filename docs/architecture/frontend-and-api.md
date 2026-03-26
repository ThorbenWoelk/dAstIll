# Frontend and API

## Product Frontend Routes

The SvelteKit app currently exposes the following top-level product routes:

| Route             | Purpose                                                                 |
| ----------------- | ----------------------------------------------------------------------- |
| `/`               | Main workspace for channels, videos, summaries, transcripts, and search |
| `/download-queue` | Queue-oriented operational view                                         |
| `/highlights`     | Cross-video highlight browser                                           |
| `/chat`           | RAG conversations with video content                                    |
| `/channels/[id]`  | Channel overview and channel-scoped operations                          |
| `/login`          | Operator sign-in                                                        |
| `/logout`         | Operator sign-out                                                       |

## Main Workspace Behavior

The main route is responsible for most user-facing behavior:

- channel selection
- video list filters
- transcript / summary / info switching
- search UI
- channels-first startup and refresh logic

## Startup Pattern

The main workspace now prioritizes first paint responsiveness:

1. load the subscribed channel list first
2. render the sidebar and current channel selection immediately
3. fetch the selected channel snapshot right after render
4. hydrate transcript / summary content once the selected video is known

That keeps the channel list off the critical path for the heavier snapshot payload.

The backend still exposes a combined convenience endpoint:

```text
GET /api/workspace/bootstrap
```

That payload includes:

- AI availability / indicator status
- channel list
- selected channel id
- initial channel snapshot
- search status

It is still useful for combined consumers and tests, but the product frontend no longer depends on it for first render.

## Important API Areas

### Channels

- list subscribed channels
- subscribe / update / delete channels
- fetch channel snapshots
- refresh and backfill channels

### Videos

- list per-channel videos
- fetch video info
- update acknowledged state

### Content

- fetch transcript
- fetch summary
- clean transcript formatting
- manually update transcript or summary
- regenerate summary

### Highlights

- create
- list by video
- list grouped views
- delete

### Search

- search content
- inspect search status
- rebuild the derived search projection

### Chat

- list conversations
- create / update / delete conversations
- stream AI responses via server-sent events
- cancel in-progress message generation
- reconnect to ongoing streams

## Handler Layer Boundaries

The backend handler modules are split by concern:

- `channels.rs` - channel CRUD, sync, refresh, backfill
- `videos.rs` - video listing, video info retrieval and enrichment
- `content.rs` - transcripts, summaries, AI health status
- `highlights.rs` - highlight CRUD
- `search.rs` - search queries, status, rebuilds
- `chat.rs` - conversations, message streaming, RAG context retrieval
- `query.rs` - shared query parameter types (filters, pagination)

The handlers are thin orchestration points. Durable logic primarily lives in:

- `db/*`
- `services/*`
- `workers.rs`

## Frontend-to-Backend Contract Style

The UI and backend communicate with typed JSON payloads rather than GraphQL or server actions. The product frontend centralizes request logic in `frontend/src/lib/api.ts`.

## Search UI Pattern

The search UI is global to the workspace, not scoped to a single video panel. It:

- uses debounced query submission
- supports source filtering (`all`, `summary`, `transcript`)
- opens results into the existing content views
- shows indexing coverage from `search_status`
