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
  ui[UI]
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
  auth[Auth + mobile handoff APIs]
  user[User state APIs]
  analytics[Analytics ingest APIs]

  ui --> library
  ui --> search
  ui --> chat
  ui --> auth
  ui --> user
  ui --> analytics
`;
</script>

## API Design

Web and native mobile clients call the backend directly through the configured API base. No Backend for Frontend (BFF).

<MermaidDiagram
  caption="Route components call the shared API client, which reaches Axum handlers. Handlers delegate durable work to storage, services, and workers."
  :chart="frontendBoundaryDiagram"
/>

Most frontend HTTP requests go through `frontend/src/lib/api.ts`, which wraps the shared transport
helpers in `frontend/src/lib/api-client.ts`.

Search-status updates use a native `EventSource` stream from `/api/search/status/stream`.

Chat replies use server-sent events over `fetch` as it supports authenticated requests,
streaming `POST` responses, cancellation, and reconnect/resume for an active conversation.

## Routing

| Route             | Purpose                                             |
| ----------------- | --------------------------------------------------- |
| `/`               | Main workspace                                      |
| `/channels/[id]`  | Per-channel overview and management                 |
| `/download-queue` | Queue-oriented operational view                     |
| `/highlights`     | Cross-video highlight browser                       |
| `/mini`           | Text-first reader for summaries and source content  |
| `/chat`           | RAG conversations                                   |
| `/vocabulary`     | Custom word replacements for summaries              |
| `/login`          | Sign-in, guest continuation, mobile browser handoff |
| `/logout`         | Session sign-out                                    |

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

## Request Trust

<MermaidDiagram
  caption="Product clients authenticate directly with Firebase bearer tokens. Trusted first-party callers can use the proxy-auth header path."
  :chart="requestTrustDiagram"
/>

The backend accepts two trust modes:

| Mode          | Inputs                                                                 | Used by                        |
| ------------- | ---------------------------------------------------------------------- | ------------------------------ |
| Direct auth   | `Authorization: Bearer <firebase-id-token>`                            | Browser frontend and Tauri UI  |
| Trusted proxy | `x-dastill-proxy-auth` plus `x-dastill-auth-state`, role, and user ids | Trusted first-party automation |

Every protected request resolves an `AccessContext` before channel, video, search, chat, or
operator-only authorization decisions.

Signed-out browsing is allowed where routes support it. Signed-out chat uses the ephemeral path and
does not write persistent conversation records.

## API Families

<MermaidDiagram
  caption="User-facing request families stay separated by concern: library/content APIs, search APIs, chat SSE APIs, auth/mobile-handoff APIs, user-state APIs, and analytics ingest APIs terminate at distinct handler boundaries."
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

### User State

- highlight listing, creation, and deletion
- route-level highlight grouping
- user preference loading and saving

### Analytics

- bounded frontend analytics event ingest

## Backend Handlers

Backend handler modules group routes by concern:

| Handler          | Routes                                           |
| ---------------- | ------------------------------------------------ |
| `auth.rs`        | Android mobile-auth handoff session lifecycle    |
| `channels.rs`    | channel CRUD, sync, refresh, backfill, bootstrap |
| `videos.rs`      | video listing, video info, acknowledged state    |
| `content.rs`     | transcripts, summaries, summary audio, AI health |
| `highlights.rs`  | highlight CRUD                                   |
| `preferences.rs` | user preferences                                 |
| `analytics.rs`   | analytics event ingest                           |
| `search.rs`      | search queries, status, status stream, rebuild   |
| `chat.rs`        | conversations, message streaming, RAG retrieval  |
| `query.rs`       | shared filter and pagination query parameters    |

Handlers orchestrate request-level work. Durable logic primarily lives in:

- `db/*`
- `services/*`
- `workers.rs`

## Contract Style

The frontend and backend communicate through typed JSON payloads plus SSE streams.

The backend OpenAPI document for local debugging can be found in `/api/openapi.json`.
The checked-in `backend/openapi.postman.yaml` file is a snapshot artifact that might be stale.
