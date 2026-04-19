---
aside: false
---

# Frontend and API

<script setup>
const frontendBoundaryDiagram = String.raw`
flowchart TB
  routes[Workspace routes]
  api[Shared API client]
  handlers[Axum handlers]
  services[db + services + workers]

  routes --> api
  api --> handlers
  handlers --> services
`;

const workspaceBootstrapDiagram = String.raw`
sequenceDiagram
  participant ui as workspace route
  participant api as /api/workspace/bootstrap
  participant state as sidebar + content state
  participant snapshot as /api/channels/{id}/snapshot
  participant content as transcript/summary loaders

  ui->>api: GET workspace bootstrap
  api-->>ui: channels + selected_channel_id + snapshot + ai/search status
  ui->>state: Render sidebar and restore selection
  alt bootstrap already includes selected snapshot
    ui->>state: Apply snapshot immediately
  else snapshot missing or stale
    ui->>snapshot: GET selected channel snapshot
    snapshot-->>ui: channel snapshot payload
    ui->>state: Apply snapshot
  end
  ui->>content: Load transcript/summary/info for selected video
`;

const requestTrustDiagram = String.raw`
flowchart TB
  browser[Browser]
  tauri[Tauri Android]
  ui[Product UI]
  direct[Firebase bearer token]
  proxy[Trusted proxy headers]
  backend[Axum backend]
  scope[AccessContext]
  authz[Scoped access]

  browser --> ui
  tauri --> ui
  ui --> direct
  direct --> backend
  proxy --> backend
  backend --> scope
  scope --> authz
`;

const apiFamiliesDiagram = String.raw`
flowchart TD
  ui[Workspace UI]
  library[Library + content APIs]
  search[Search APIs]
  chat[Chat + SSE APIs]
  user[Highlights + preferences]

  ui --> library
  ui --> search
  ui --> chat
  ui --> user
`;
</script>

## Product Frontend Routes

The SvelteKit app currently exposes the following top-level product routes:

| Route             | Purpose                                                                 |
| ----------------- | ----------------------------------------------------------------------- |
| `/`               | Main workspace for channels, videos, summaries, transcripts, and search |
| `/channels/[id]`  | Dedicated per-channel overview and management view                      |
| `/download-queue` | Queue-oriented operational view                                         |
| `/highlights`     | Cross-video highlight browser                                           |
| `/mini`           | Text-first reader for summaries and source content                      |
| `/chat`           | RAG conversations with video content                                    |
| `/vocabulary`     | Manage custom word replacements for summaries                           |
| `/login`          | Firebase sign-in and guest continuation                                 |
| `/logout`         | Session sign-out                                                        |

<MermaidDiagram
  caption="Frontend boundary: route components call the shared API client, which fans out into handler modules that delegate durable logic to db, services, and workers."
  :chart="frontendBoundaryDiagram"
/>

## Main Workspace Behavior

The main route is responsible for most user-facing behavior:

- channel selection
- video list filters
- transcript / summary / info switching
- summary-audio playback when TTS is configured
- search UI
- workspace bootstrap, selection restore, and refresh logic

The `/mini` route is a separate reading surface for the same saved content. Its responsive architecture is documented in [Mini Reader](/architecture/mini-reader).

## Startup Pattern

The main workspace uses a combined bootstrap response:

1. request `/api/workspace/bootstrap` during the route load
2. receive the channel list, selected channel id, AI/search status, and an initial snapshot when available
3. render the sidebar and apply the selected snapshot immediately
4. fall back to a snapshot fetch only if the bootstrap response does not include a usable one
5. hydrate transcript / summary content once the selected video is known

This keeps the initial workspace state coherent while still allowing the deeper snapshot fetch to be retried when needed.

<MermaidDiagram
  caption="Workspace bootstrap flow: the frontend asks for the channel list plus an optional selected-channel snapshot, then hydrates deeper content after the selection is known."
  :chart="workspaceBootstrapDiagram"
/>

## Request Trust Model

The current frontend no longer depends on a SvelteKit API proxy route layer for normal product traffic.

- Browser and Tauri clients call the Rust backend directly using `VITE_API_BASE`.
- Signed-in direct requests send `Authorization: Bearer <firebase-id-token>`.
- Trusted first-party callers can still use `x-dastill-proxy-auth` plus `x-dastill-*` headers when they need proxy-style identity forwarding.
- The backend accepts either mode and always resolves an `AccessContext` before channel, video, search, or chat authorization decisions.

<MermaidDiagram
  caption="Request trust modes: normal product clients authenticate directly with Firebase bearer tokens, while trusted first-party callers can still use the proxy-auth header path."
  :chart="requestTrustDiagram"
/>

The backend exposes a combined convenience endpoint:

```text
GET /api/workspace/bootstrap
```

The payload includes:

- AI availability / indicator status
- channel list
- selected channel id
- initial channel snapshot
- search status

This endpoint is useful for combined consumers and tests. The product frontend uses it as the primary SSR/bootstrap path and only falls back to a later selected-channel snapshot fetch when the bootstrap payload does not include one.

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
- generate and fetch cached summary audio
- clean transcript formatting
- manually update transcript or summary
- regenerate summary

### Auth and Session Handoff

- create, poll, complete, and delete Android mobile-auth handoff sessions
- support system-browser Google sign-in for the Tauri Android shell

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
- allow a per-message deep-research flag to widen retrieval budgets
- signed-in users use persistent conversations; signed-out visitors use the ephemeral chat path

### Analytics

- ingest frontend analytics events
- batch submission with size limits
- queues events for downstream processing

<MermaidDiagram
  caption="User-facing request families stay separated by concern: library and content APIs, search APIs, and chat SSE APIs all terminate at distinct handler boundaries."
  :chart="apiFamiliesDiagram"
/>

## Handler Layer Boundaries

The backend handler modules are split by concern:

- `auth.rs` - Android mobile-auth handoff session lifecycle
- `channels.rs` - channel CRUD, sync, refresh, backfill
- `videos.rs` - video listing, video info retrieval and enrichment
- `content.rs` - transcripts, summaries, summary audio, AI health status
- `highlights.rs` - highlight CRUD
- `search.rs` - search queries, status, rebuilds
- `chat.rs` - conversations, message streaming, RAG context retrieval
- `query.rs` - shared query parameter types (filters, pagination)

The handlers are thin orchestration points. Durable logic primarily lives in:

- `db/*`
- `services/*`
- `workers.rs`

## Frontend-to-Backend Contract Style

The UI and backend communicate with typed JSON payloads plus SSE streams rather than GraphQL or server actions. The product frontend centralizes request logic in `frontend/src/lib/api.ts`, while chat and search status streams use `EventSource`.

For manual backend debugging, the backend also exposes a live OpenAPI document at
`/api/openapi.json`. During local development, Postman should import the running backend URL
instead of relying on the checked-in `backend/openapi.postman.yaml` snapshot.

## Search UI Pattern

The search UI is global to the workspace, not scoped to a single video panel. It:

- uses debounced query submission
- supports source filtering (`all`, `summary`, `transcript`)
- opens results into the existing content views
- shows indexing coverage from `search_status`
