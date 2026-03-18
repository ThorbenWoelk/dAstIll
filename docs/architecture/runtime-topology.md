# Runtime Topology

## Process Model

In active development, dAstIll typically runs as three separate processes:

```text
1. frontend/ SvelteKit dev server
2. backend/ Rust API + worker host
3. docs/ VitePress dev server
```

Only the backend process owns durable state changes and worker execution.

## Backend Startup Sequence

At startup the backend:

```text
1. Loads backend/.env if present
2. Configures AWS SDK with local credentials or GCP Workload Identity Federation
3. Connects to S3 data bucket and S3 Vectors bucket
4. Initializes the store (no schema migrations needed - S3 is schemaless)
5. Hydrates search progress from existing data
6. Builds shared runtime services
7. Spawns background workers
8. Binds the Axum HTTP listener
```

## Shared Runtime State

`AppState` carries the core runtime singletons:

- S3 store (data + vectors clients)
- read cache
- search projection lock
- search progress tracker
- YouTube service
- transcript service
- summarizer service
- summary evaluator service
- search service
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
