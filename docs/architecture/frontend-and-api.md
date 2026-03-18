# Frontend and API

## Product Frontend Routes

The SvelteKit app currently exposes three top-level routes:

| Route             | Purpose                                                                 |
| ----------------- | ----------------------------------------------------------------------- |
| `/`               | Main workspace for channels, videos, summaries, transcripts, and search |
| `/download-queue` | Queue-oriented operational view                                         |
| `/highlights`     | Cross-video highlight browser                                           |

## Main Workspace Behavior

The main route is responsible for most user-facing behavior:

- channel selection
- video list filters
- transcript / summary / info switching
- search UI
- workspace bootstrap and refresh logic

## Bootstrap Pattern

The frontend does **not** reconstruct workspace state from many small requests on first paint. It uses a combined bootstrap request:

```text
GET /api/workspace/bootstrap
```

That payload includes:

- AI availability / indicator status
- channel list
- selected channel id
- initial channel snapshot
- search status

This keeps the workspace from stitching together too many independent startup requests.

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

## Handler Layer Boundaries

The backend handler modules are split by concern:

- `channels.rs`
- `videos.rs`
- `video_info.rs`
- `content.rs`
- `highlights.rs`
- `search.rs`
- `query.rs`

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
