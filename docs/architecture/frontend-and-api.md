# Frontend and API

<script setup>
const frontendBoundaryDiagram = String.raw`
flowchart TB
  routes[Reader routes]
  edition[Edition state]
  api[API client]
  handlers[Axum handlers]
  services[db + services + workers]

  routes --> edition
  edition --> api
  routes --> api
  api --> handlers
  handlers --> services
`;

const editionLoadDiagram = String.raw`
sequenceDiagram
  participant ui as reader page
  participant cache as browser storage
  participant api as /api/mini
  participant read as /api/mini/videos/{id}/read

  ui->>cache: show last stored edition
  ui->>api: GET (no channel id)
  api-->>ui: channel list + first channel's unread summaries
  par at most four at a time
    ui->>api: GET ?channel_id=...
    api-->>ui: that channel's unread summaries
  end
  ui->>cache: store merged edition
  ui->>read: PUT {read: true} when a story is finished
`;

const requestTrustDiagram = String.raw`
flowchart TB
  browser[Browser]
  ui[Reader UI]
  direct[Firebase bearer token]
  proxy[Trusted proxy headers]
  backend[Axum backend]
  scope[AccessContext]
  authz[Scoped access]

  browser --> ui
  ui --> direct
  direct --> backend
  proxy --> backend
  backend --> scope
  scope --> authz
`;

const apiFamiliesDiagram = String.raw`
flowchart TD
  ui[Reader UI]
  mini[Mini reader APIs]
  library[Channel APIs]
  highlights[Highlight APIs]
  other[Search, chat, analytics APIs]

  ui --> mini
  ui --> library
  ui --> highlights
  other -.-> none[No UI in the current reader]
`;
</script>

## API Design

The reader calls the backend directly through the configured API base (`PUBLIC_API_BASE`).
There is no Backend for Frontend (BFF).

<MermaidDiagram
  caption="Routes read state from the edition controller, which calls the API client. Handlers delegate durable work to storage, services, and workers."
  :chart="frontendBoundaryDiagram"
/>

## Frontend Structure

The frontend is a static SvelteKit single-page app. It renders entirely in the browser.

| Path                                            | Purpose                                                      |
| ----------------------------------------------- | ------------------------------------------------------------ |
| `src/lib/api.ts`                                | All backend requests, token header, readable errors          |
| `src/lib/session.svelte.ts`                     | Firebase Google sign-in state                                |
| `src/lib/edition/printing.ts`                   | Loads every channel and merges stories by release date       |
| `src/lib/edition/stories.ts`                    | Pure story, section, and date helpers                        |
| `src/lib/edition/summary.ts`                    | Splits a summary into standfirst, glance box, and body       |
| `src/lib/edition/markdown.ts`                   | Markdown to sanitized HTML                                   |
| `src/lib/edition/keepsake.ts`                   | Last edition and chosen section in browser storage           |
| `src/lib/edition/reader.svelte.ts`              | Reader state: sections, lead story, mark read, undo          |
| `src/lib/highlights/anchoring.ts`               | Builds a highlight from a selection and finds it again       |
| `src/lib/highlights/passage.ts`                 | Story text as one string; draws and clears highlight marks   |
| `src/lib/highlights/story-highlights.svelte.ts` | Highlights of the story on screen, optimistic add and remove |
| `src/lib/highlights/collection.ts`              | Search and remove on the grouped highlights list             |
| `src/lib/components/`                           | Masthead, section nav, story, read bar, rail, sign-in        |
| `src/lib/bindings/`                             | Types generated from the backend by `ts-rs`                  |

## Routing

| Route           | Purpose                                                     |
| --------------- | ----------------------------------------------------------- |
| `/`             | The reader: front page or one section                       |
| `/highlights`   | Every highlight by story, newest stories first, with search |
| `/finished`     | Stories marked as read, most recently finished first        |
| `/stories/{id}` | One story, read or unread, with its highlights              |
| `/sections`     | Follow a new channel or stop following one; sign out        |
| `/mini`         | Old reader URL. Firebase Hosting redirects it to `/`.       |

Signed-out visitors see the sign-in page on every route.

## Loading The Paper

`GET /api/mini` returns unread summaries for one channel at a time. The reader asks once without a
channel id, which returns the channel list and the first channel's summaries, then asks for the
other channels with at most four requests in flight. It merges the results newest first.

<MermaidDiagram
  caption="The reader shows the stored edition at once, then loads every channel and stores the merged result. Finishing a story sends one small PUT."
  :chart="editionLoadDiagram"
/>

Marking a story read updates the page first and then sends the request. If the request fails, the
story comes back and a notice explains why. Undo sends `read: false` after the first request has
finished, so the two never race.

The reader reloads only when asked, or when the tab becomes visible again after 30 minutes. See
[Running Cost For One Reader](/operations/deployment#running-cost-for-one-reader).

## Request Trust

<MermaidDiagram
  caption="Product clients authenticate directly with Firebase bearer tokens. Trusted first-party callers can use the proxy-auth header path."
  :chart="requestTrustDiagram"
/>

The backend accepts two trust modes:

| Mode          | Inputs                                                                 | Used by                        |
| ------------- | ---------------------------------------------------------------------- | ------------------------------ |
| Direct auth   | `Authorization: Bearer <firebase-id-token>`                            | Reader frontend                |
| Trusted proxy | `x-dastill-proxy-auth` plus `x-dastill-auth-state`, role, and user ids | Trusted first-party automation |

Every protected request resolves an `AccessContext` before channel, video, search, chat, or
operator-only authorization decisions.

The reader requires Google sign-in. The backend still supports signed-out access on routes that
allow it, such as the ephemeral chat path.

## API Families

<MermaidDiagram
  caption="The reader uses the mini reader, channel, and highlight APIs. The other API families remain in the backend without a UI."
  :chart="apiFamiliesDiagram"
/>

The reader uses:

- `GET /api/mini` and `PUT /api/mini/videos/{id}/read`
- `GET /api/mini/videos/{id}`: one story from a followed channel, read or unread
- `GET /api/mini/finished?offset&limit`: finished stories, most recently finished first, with
  `finished_at` and `has_more`
- `GET /api/channels`, `POST /api/channels`, and `DELETE /api/channels/{id}`
- `GET /api/highlights`, `GET` and `POST /api/videos/{id}/highlights`, and
  `DELETE /api/highlights/{id}`. Highlight ids are sent as strings because they are larger than
  JavaScript's safe integers.

Read state per reader is cached in memory on the backend for 30 minutes and updated on every
write, so loading the paper and the finished list does not read one storage object per story.

The families below are still served by the backend. The current frontend does not call them.

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
