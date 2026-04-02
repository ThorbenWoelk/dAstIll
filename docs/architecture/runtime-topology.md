# Runtime Topology

<script setup>
const processModelDiagram = String.raw`
flowchart LR
  browser[Browser]
  frontend[frontend/<br/>SvelteKit dev server]
  docs[docs/<br/>VitePress dev server]
  backend[backend/<br/>Axum API + worker host]
  appstate[AppState shared services]

  subgraph workers["Long-lived backend tasks"]
    queue[Queue worker]
    refresh[Refresh worker]
    gap[Gap scan worker]
    eval[Summary evaluation worker]
    search[Search index worker]
    fts[FTS hydration task]
  end

  browser --> frontend
  browser --> docs
  frontend --> backend
  backend --> appstate
  appstate --> queue
  appstate --> refresh
  appstate --> gap
  appstate --> eval
  appstate --> search
  appstate --> fts
`;

const startupSequenceDiagram = String.raw`
sequenceDiagram
  participant boot as backend main
  participant store as S3 + S3 Vectors + Firestore
  participant state as AppState
  participant bg as background tasks
  participant http as Axum router

  boot->>store: Initialize storage clients
  boot->>state: Build runtime services and shared state
  boot->>bg: Hydrate search progress
  boot->>bg: Hydrate keyword index if libSQL/Turso is empty
  boot->>bg: Spawn queue, refresh, gap, eval, and search workers
  boot->>http: Register routes and bind listener
  http-->>boot: Ready for frontend requests
`;

const concurrencyDiagram = String.raw`
flowchart TD
  appstate[AppState]
  lock[search_projection_lock]
  cooldowns[Cooldowns + semaphores]
  queue[Queue worker]
  search[Search index worker]
  eval[Summary evaluation worker]
  chat[Chat service]
  http[HTTP handlers]

  appstate --> queue
  appstate --> search
  appstate --> eval
  appstate --> chat
  appstate --> http
  search --> lock
  http --> lock
  queue --> cooldowns
  eval --> cooldowns
  chat --> cooldowns
`;
</script>

## Process Model

In active development, dAstIll typically runs as three separate processes:

```text
1. frontend/ SvelteKit dev server
2. backend/ Rust API + worker host
3. docs/ VitePress dev server
```

Only the backend process owns durable state changes and worker execution.

<MermaidDiagram
  caption="The frontend and docs run as separate processes, while the backend owns shared runtime state, workers, and all durable writes."
  :chart="processModelDiagram"
/>

## Backend Startup Sequence

At startup the backend:

```text
1. Loads backend/.env if present
2. Configures AWS SDK with local credentials or GCP Workload Identity Federation
3. Connects to S3 data bucket and S3 Vectors bucket
4. Initializes the mixed S3 / S3 Vectors / Firestore store layer
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
- search projection lock
- search progress tracker
- **FTS index** (libSQL/Turso BM25 index; shared `Arc<RwLock<_>>`)
- chat service
- active chats tracker (in-progress conversations)
- chat store lock
- YouTube service
- transcript service
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
- video browsing
- transcript and summary editing
- highlights
- search

### Docs UI

Serves technical documentation only and has no dependency on the product frontend runtime.
