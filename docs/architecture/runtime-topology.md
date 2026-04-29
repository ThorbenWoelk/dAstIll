---
aside: false
---

# Runtime Topology

<script setup>
const processModelDiagram = String.raw`
flowchart TB
  browser[Browser]
  frontend[frontend/<br/>SvelteKit dev server]
  docs[docs/<br/>VitePress dev server]
  backend[backend/<br/>Axum API + worker host]
  asr[optional local ASR<br/>whisper.cpp server]
  android[optional Tauri<br/>Android shell]
  appstate[AppState]
  workers[Worker loops]

  browser --> frontend
  browser --> docs
  frontend --> backend
  backend -. podcast audio .-> asr
  android -. adb reverse .-> frontend
  android -. adb reverse .-> backend
  backend --> appstate
  appstate --> workers
`;

const startupSequenceDiagram = String.raw`
sequenceDiagram
  participant boot as Boot
  participant listener as TCP listener
  participant store as Storage
  participant state as AppState
  participant hydrate as Hydration tasks
  participant workers as Workers
  participant http as HTTP

  boot->>listener: bind PORT early
  boot->>store: init S3, S3 Vectors, and local libSQL
  boot->>store: restore/reconcile/publish SQL snapshot
  boot->>state: build shared services
  boot->>hydrate: start search progress + FTS hydration
  boot->>workers: start worker loops
  boot->>http: build routes + serve listener
  http-->>boot: ready
`;

const concurrencyDiagram = String.raw`
flowchart TD
  appstate[AppState]
  http[HTTP handlers]
  workers[Queue, eval,<br/>and search workers]
  chat[Chat service]
  lock[Projection lock]
  limits[Rate limits,<br/>cooldowns + semaphores]

  appstate --> http
  appstate --> workers
  appstate --> chat
  http --> lock
  workers --> lock
  workers --> limits
  chat --> limits
`;
</script>

## Process Model

In active development, dAstIll typically runs as three separate processes:

```text
1. backend/ Rust API + worker host
2. frontend/ SvelteKit dev server
3. docs/ VitePress dev server
```

`./start_app.sh` starts the frontend with `VITE_API_BASE` pointed at the local backend, so local `/api` traffic is proxied from the SvelteKit dev server to Axum.

Optional processes can also appear:

```text
4. Local ASR server (when LOCAL_ASR_ENABLED=true and LOCAL_ASR_BASE_URL is localhost/127.0.0.1)
5. Tauri Android shell (when START_APP_MOBILE=1 after the local services are healthy)
```

Only the backend process owns durable state changes and worker execution. The local ASR process is a transient transcript dependency for podcast audio.

`LOCAL_APP_MAINTENANCE_MODE=1` is a frontend maintenance preview. In that mode `./start_app.sh` starts the frontend and docs, then skips backend startup.

<MermaidDiagram
  caption="The frontend and docs run as separate processes, while the backend owns shared runtime state, workers, and all durable writes. Optional local ASR and Android shell processes sit outside that ownership boundary."
  :chart="processModelDiagram"
/>

## Backend Startup Sequence

At startup the backend:

```text
1. Loads env with this effective precedence: shell, then `backend/.env`, then `~/.config/dastill/backend.env`
2. Applies shared local AWS credential/config file defaults when present
3. Configures logging and binds the TCP listener for `PORT` early
4. Reads runtime config for search, chat, Databricks analytics, Polly TTS, local ASR, security, and Ollama
5. Configures AWS SDK with the default provider chain or GCP Workload Identity Federation
6. Connects to S3 data bucket and S3 Vectors bucket
7. Restores the local libSQL snapshot from S3, initializes schema, builds the store, reconciles SQL cache rows, and may publish a fresh snapshot
8. Builds shared runtime services and `AppState`
9. Spawns search progress hydration
10. If the keyword index is empty, spawns FTS hydration from search bundles, legacy search chunks, or raw materials
11. Spawns background workers
12. Builds the Axum router and serves the already-bound listener
```

<MermaidDiagram
  caption="Startup flow: bind the backend port first, initialize storage and services, hydrate search state in background tasks, start worker loops, then serve HTTP routes."
  :chart="startupSequenceDiagram"
/>

## Shared Runtime State

`AppState` carries the core runtime singletons:

- mixed S3 / S3 Vectors / local libSQL store
- read cache
- security/runtime auth config
- request rate limiter
- search auto-create-vector-index flag
- search projection lock
- search progress tracker
- FTS index (local libSQL BM25 index)
- chat service
- input guardrail service
- optional Databricks analytics sink
- active chats tracker (in-progress conversations)
- chat store lock
- anonymous chat quota lock
- mobile auth handoff sessions
- YouTube service
- OpenAlex planner and OpenAlex service
- podcast feed service
- website ingestion service
- transcript service
- optional Polly TTS service
- summarizer service
- summary evaluator service
- search service (embedding, reranker, HyDE)
- cloud, YouTube quota, and transcript cooldown trackers
- user activity tracker

This is the boundary between HTTP handlers and long-lived background processes.

## Parallel Worker Loops

The backend starts five worker loops in parallel.

### Queue Worker

Purpose:

- advances per-video transcript and summary generation

Behavior:

- polls every 5 seconds after active work
- backs off from 15 seconds to 60 seconds while idle
- prioritizes transcript before summary
- increments retry counts on non-rate-limit failures
- pauses transcript work during transcript dependency cooldown
- skips work when there is no recent active user

### Refresh Worker

Purpose:

- fetches latest videos for all subscribed channels

Behavior:

- performs an initial refresh at startup
- runs every 30 minutes afterward
- skips scheduled refreshes when there is no recent active user

### Gap Scan Worker

Purpose:

- backfills missing historical videos

Behavior:

- runs every 10 minutes
- respects YouTube quota cooldown
- scans a bounded number of videos per channel each round
- skips work when there is no recent active user

### Summary Evaluation Worker

Purpose:

- scores summaries against transcripts
- queues low-quality summaries for regeneration

Behavior:

- polls every 7 seconds after active work
- backs off from 30 seconds to 120 seconds while idle
- only runs when evaluator policy permits
- queues low-quality summaries for automatic regeneration when retry policy allows
- skips work when there is no recent active user

### Search Index Worker

Purpose:

- backfills missing search sources
- indexes pending transcript and summary content
- reconciles stale sources
- prunes stale rows
- optionally creates the ANN vector index

Behavior:

- performs an initial backfill, reconcile, prune, and vector-index check before the loop
- polls every 3 seconds after active work
- backs off from 15 seconds to 120 seconds while idle
- reconciles every 60 seconds
- logs indexing rounds with batch and embedding counts
- retries vector-index creation at most every 5 minutes
- skips work when there is no recent active user

## Concurrency Controls

<MermaidDiagram
  caption="Concurrency boundaries: workers share AppState, search projection changes coordinate through a read/write lock, and model-heavy paths are rate-limited separately."
  :chart="concurrencyDiagram"
/>

### Projection lock

Search rebuilds and index maintenance coordinate through a `RwLock` so destructive resets and normal search/index reads do not stomp on each other.

### Local model semaphores

The summarizer/evaluator side and the search embedding side each use a separate semaphore to keep local-model concurrency bounded.

### Runtime throttles

HTTP handlers share a request rate limiter from `AppState`. Cloud-model calls, YouTube quota failures, and transcript dependency failures each have their own cooldown tracker.

## User-Facing Frontends

### Product UI

Serves interactive workspace features:

- channel management
- per-channel overview and sync controls
- video browsing
- transcript and summary editing
- summary-audio playback when TTS is enabled
- highlights
- search
- library chat
- a browser-only service worker / PWA shell for cached static assets, API GET responses, and thumbnails

### Docs UI

Serves technical documentation only and has no dependency on the product frontend runtime.
