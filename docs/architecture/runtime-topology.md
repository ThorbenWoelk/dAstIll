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
  appstate[AppState]
  workers[Worker loops]

  browser --> frontend
  browser --> docs
  frontend --> backend
  backend --> appstate
  appstate --> workers
`;

const startupSequenceDiagram = String.raw`
sequenceDiagram
  participant boot as Boot
  participant store as Storage
  participant state as AppState
  participant workers as Workers
  participant http as HTTP

  boot->>store: init storage clients
  boot->>state: build shared runtime
  boot->>workers: hydrate search + FTS
  boot->>workers: start worker loops
  boot->>http: bind routes + listener
  http-->>boot: ready
`;

const concurrencyDiagram = String.raw`
flowchart TD
  appstate[AppState]
  http[HTTP handlers]
  workers[Queue, eval,<br/>and search workers]
  chat[Chat service]
  lock[Projection lock]
  limits[Cooldowns + semaphores]

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
1. frontend/ SvelteKit dev server
2. backend/ Rust API + worker host
3. docs/ VitePress dev server
```

When you use `./start_app.sh` with an Android device or emulator connected, an optional fourth process can also appear:

```text
4. Tauri Android shell (opt-in launch after the local services are healthy)
```

Only the backend process owns durable state changes and worker execution.

<MermaidDiagram
  caption="The frontend and docs run as separate processes, while the backend owns shared runtime state, workers, and all durable writes."
  :chart="processModelDiagram"
/>

## Backend Startup Sequence

At startup the backend:

```text
1. Loads shell env first, then `~/.config/dastill/backend.env`, then `backend/.env` if present
2. Configures AWS SDK with local credentials or GCP Workload Identity Federation
3. Connects to S3 data bucket and S3 Vectors bucket
4. Initializes the mixed S3 / S3 Vectors / libSQL-Turso store layer
5. Hydrates search progress from existing data
6. Builds shared runtime services (including the libSQL/Turso keyword index)
7. Spawns background workers
8. If the keyword index is empty, spawns FTS hydration task: concurrently loads
   all search bundles/chunks into the libSQL/Turso index so keyword search is available
9. Binds the Axum HTTP listener
```

<MermaidDiagram
  caption="Startup flow: initialize storage and services first, then hydrate search state, then start background loops, then accept HTTP traffic."
  :chart="startupSequenceDiagram"
/>

## Shared Runtime State

`AppState` carries the core runtime singletons:

- S3 store (data + vectors clients)
- read cache
- security/runtime auth config
- search projection lock
- search progress tracker
- **FTS index** (libSQL/Turso BM25 index; shared `Arc<RwLock<_>>`)
- chat service
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
- cooldown trackers

This is the boundary between HTTP handlers and long-lived background processes.

## Parallel Worker Loops

The backend starts five worker loops in parallel.

### Queue Worker

Purpose:

- advances per-video transcript and summary generation

Behavior:

- polls every 5 seconds
- prioritizes transcript before summary
- increments retry counts on non-rate-limit failures

### Refresh Worker

Purpose:

- fetches latest videos for all subscribed channels

Behavior:

- performs an initial refresh at startup
- runs every 30 minutes afterward

### Gap Scan Worker

Purpose:

- backfills missing historical videos

Behavior:

- runs every 10 minutes
- respects YouTube quota cooldown
- scans a bounded number of videos per channel each round

### Summary Evaluation Worker

Purpose:

- scores summaries against transcripts
- queues low-quality summaries for regeneration

Behavior:

- polls every 7 seconds
- only runs when evaluator policy permits

### Search Index Worker

Purpose:

- backfills missing search sources
- indexes pending transcript and summary content
- reconciles stale sources
- prunes stale rows
- optionally creates the ANN vector index

Behavior:

- polls every 3 seconds
- reconciles on a longer cadence
- logs indexing rounds with batch and embedding counts

## Concurrency Controls

<MermaidDiagram
  caption="Concurrency boundaries: workers share AppState, search projection changes coordinate through a read/write lock, and model-heavy paths are rate-limited separately."
  :chart="concurrencyDiagram"
/>

### Projection lock

Search rebuilds and index maintenance coordinate through a `RwLock` so destructive resets and normal search/index reads do not stomp on each other.

### Local model semaphores

The summarizer/evaluator side and the search embedding side each use a separate semaphore to keep local-model concurrency bounded.

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
