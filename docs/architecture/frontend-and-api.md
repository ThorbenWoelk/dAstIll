# Frontend and API

<script setup>
const frontendBoundaryDiagram = String.raw`
flowchart TB
  routes[Product routes]
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
  api-->>ui: channels + selected ids + snapshot + ai/search status
  ui->>state: render sidebar and restore selection
  alt bootstrap includes selected snapshot
    ui->>state: apply snapshot immediately
  else snapshot missing or stale
    ui->>snapshot: GET selected channel snapshot
    snapshot-->>ui: channel snapshot payload
    ui->>state: apply snapshot
  end
  ui->>content: load transcript/summary/info for selected video
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

## Boundary

<MermaidDiagram
  caption="Route components call the shared API client, which reaches Axum handlers. Handlers delegate durable work to storage, services, and workers."
  :chart="frontendBoundaryDiagram"
/>

The product frontend does not use a SvelteKit API route layer for normal backend traffic. Browser and
Tauri clients call the Rust backend directly through the configured API base.

Frontend request code is centralized in `frontend/src/lib/api.ts`. Chat and search-status live
updates use `EventSource`.

## Product Routes

| Route             | Purpose                                                      |
| ----------------- | ------------------------------------------------------------ |
| `/`               | Main workspace                                               |
| `/channels/[id]`  | Per-channel overview and management                          |
| `/download-queue` | Queue-oriented operational view                              |
| `/highlights`     | Cross-video highlight browser                                |
| `/mini`           | Text-first reader for summaries and source content           |
| `/chat`           | RAG conversations                                            |
| `/vocabulary`     | Custom word replacements for summaries                       |
| `/login`          | Firebase sign-in, guest continuation, mobile browser handoff |
| `/logout`         | Session sign-out                                             |

## Workspace Bootstrap

The main workspace starts from:

```text
GET /api/workspace/bootstrap
```

The payload includes:

- AI availability and AI status
- library containers and sources
- channel list
- selected source/channel/item ids
- initial channel snapshot when available
- search status

<MermaidDiagram
  caption="Workspace bootstrap loads the sidebar, selected ids, optional selected-channel snapshot, and status surfaces before deeper content hydration."
  :chart="workspaceBootstrapDiagram"
/>

The frontend applies a snapshot from bootstrap immediately when it is present. If the bootstrap
payload lacks a usable selected-channel snapshot, the frontend fetches
`/api/channels/{id}/snapshot` and then loads transcript, summary, and video info for the selected
video.

## View Models

API responses intentionally combine storage records into UI-ready view models.

| Response area | View-model behavior                                                             |
| ------------- | ------------------------------------------------------------------------------- |
| Channels      | canonical channel data plus caller-specific subscription state                  |
| Videos        | canonical video row plus caller-specific `acknowledged` state                   |
| Highlights    | per-user highlight records grouped for route display                            |
| Chat          | persistent conversations for signed-in users; ephemeral path for signed-out use |
| Search        | grouped video results with transcript/summary snippets and status metadata      |

Durable ownership for these records is documented in [Data Model](/architecture/data-model).

## Request Trust

<MermaidDiagram
  caption="Product clients authenticate directly with Firebase bearer tokens. Trusted first-party callers can use the proxy-auth header path."
  :chart="requestTrustDiagram"
/>

The backend accepts two trust modes:

| Mode          | Inputs                                                                 | Used by                        |
| ------------- | ---------------------------------------------------------------------- | ------------------------------ |
| Direct auth   | `Authorization: Bearer <firebase-id-token>`                            | Browser and Tauri product UI   |
| Trusted proxy | `x-dastill-proxy-auth` plus `x-dastill-auth-state`, role, and user ids | Trusted first-party automation |

Every protected request resolves an `AccessContext` before channel, video, search, chat, or
operator-only authorization decisions.

Signed-out browsing is allowed where routes support it. Signed-out chat uses the ephemeral path and
does not write persistent conversation records.

## API Families

<MermaidDiagram
  caption="User-facing request families stay separated by concern: library/content APIs, search APIs, chat SSE APIs, and user-state APIs terminate at distinct handler boundaries."
  :chart="apiFamiliesDiagram"
/>

### Library And Content

- channel list, subscribe, update, delete, refresh, and backfill
- channel snapshots and per-channel videos
- transcript, summary, video info, summary audio
- manual transcript/summary edits
- summary regeneration
- acknowledged state updates

### Search

- search content
- inspect search status
- stream search status
- rebuild the derived search projection

### Chat

- list conversations
- create, update, and delete conversations
- stream assistant responses through SSE
- cancel or reconnect to in-progress generation
- send signed-out prompts through the ephemeral path
- allow per-message deep-research retrieval expansion

### Auth And Mobile Handoff

- create, poll, complete, and delete Android mobile-auth handoff sessions
- support system-browser Google sign-in for the Tauri Android shell

### User State And Analytics

- create and delete highlights
- list highlights by video or grouped route view
- get and save preferences
- ingest frontend analytics events in bounded batches

## Handler Boundaries

Backend handler modules are split by concern:

| Handler         | Boundary                                         |
| --------------- | ------------------------------------------------ |
| `auth.rs`       | Android mobile-auth handoff session lifecycle    |
| `channels.rs`   | channel CRUD, sync, refresh, backfill, bootstrap |
| `videos.rs`     | video listing, video info, acknowledged state    |
| `content.rs`    | transcripts, summaries, summary audio, AI health |
| `highlights.rs` | highlight CRUD                                   |
| `search.rs`     | search queries, status, status stream, rebuild   |
| `chat.rs`       | conversations, message streaming, RAG retrieval  |
| `query.rs`      | shared filter and pagination query parameters    |

Handlers orchestrate request-level work. Durable logic primarily lives in:

- `db/*`
- `services/*`
- `workers.rs`

## Contract Style

The frontend and backend communicate through typed JSON payloads plus SSE streams. They do not use
GraphQL or SvelteKit server actions for product backend calls.

The live backend OpenAPI document is the local debugging source of truth:

```text
/api/openapi.json
```

The checked-in `backend/openapi.postman.yaml` file is a snapshot artifact.

## Search UI Pattern

The search UI is global to the workspace.

It:

- uses debounced query submission
- supports source filtering: `all`, `summary`, `transcript`
- opens results into existing content views
- shows indexing coverage from `search_status`
