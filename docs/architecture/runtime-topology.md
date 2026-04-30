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

## Backend Startup Sequence

```text
1. Loads env vars and local AWS defaults
2. Configures logging and binds the TCP listener for `PORT` early
3. Parse and validate runtime config from env vars for search, chat, Databricks analytics, Polly TTS, local ASR, security, and Ollama
4. Configures AWS SDK with the default provider chain or GCP Workload Identity Federation
5. Connects to S3 data bucket and S3 Vectors bucket
6. Restores the local libSQL snapshot from S3
7. Initializes libSQL schema, builds the store, reconciles SQL cache rows, and may publish a fresh snapshot
8. Builds shared runtime services and `AppState`
9. Starts search progress hydration
10. Starts FTS hydration when the keyword index is empty
11. Starts background worker loops
12. Builds the Axum router and serves the listener
```

<MermaidDiagram
  caption="Startup flow: bind the backend port first, initialize storage and services, hydrate search state in background tasks, start worker loops, then serve HTTP routes."
  :chart="startupSequenceDiagram"
/>

## Shared Runtime State

`AppState` is the boundary shared by Axum handlers, services, chat generation, and background
workers.

| Area            | `AppState` owns                                                                                       |
| --------------- | ----------------------------------------------------------------------------------------------------- |
| Storage         | mixed S3 / S3 Vectors / local libSQL store, read cache                                                |
| Security        | runtime auth config, request rate limiter, mobile auth handoff sessions                               |
| Search          | auto-create-vector-index flag, projection lock, progress tracker, FTS index, search service           |
| Chat            | chat service, input guardrail service, active chats tracker, chat store lock, anonymous chat quota    |
| Source services | YouTube service, OpenAlex planner/service, podcast feed service, website ingestion, transcript client |
| Model services  | optional Polly TTS service, summarizer service, summary evaluator service                             |
| Observability   | optional Databricks analytics sink                                                                    |
| Cooldowns       | cloud, YouTube quota, and transcript dependency trackers                                              |
| Activity gating | user activity tracker                                                                                 |

`AppState` is process-wide. The backend builds it once at startup and shares it across all
requests and worker loops.

`AccessContext` is separate request-scoped state. Auth middleware attaches it to each HTTP request.
Handlers read `AppState` for shared services and `AccessContext` for caller-specific access data.

| `AccessContext` field | Used for                                                      |
| --------------------- | ------------------------------------------------------------- |
| `user_id`             | authenticated user scope for chats, highlights, and library   |
| `auth_state`          | distinguishes authenticated and anonymous request paths       |
| `access_role`         | enables operator-only behavior                                |
| `allowed_channel_ids` | bounds visible channel scope                                  |
| `allowed_other_video_ids` | bounds explicit video access outside subscribed channels |

## Parallel Worker Loops

The backend starts five worker loops in parallel.

| Worker                    | Runtime role                                            | Cadence and guard                                                       |
| ------------------------- | ------------------------------------------------------- | ----------------------------------------------------------------------- |
| Queue worker              | advances transcript work before summary work            | polls every 5s after active work; backs off from 15s to 60s while idle  |
| Refresh worker            | fetches latest videos for subscribed channels           | runs once at startup, then every 30 minutes                             |
| Gap scan worker           | backfills missing historical videos                     | runs every 10 minutes and respects YouTube quota cooldown               |
| Summary evaluation worker | scores summaries and can queue low-quality regeneration | polls every 7s after active work; backs off from 30s to 120s while idle |
| Search index worker       | backfills, indexes, reconciles, prunes, and syncs FTS   | polls every 3s after active work; backs off from 15s to 120s while idle |

All five loops skip scheduled work when there is no recent active user. Model and external-service
failures can also activate a cooldown for the affected path.

The search index worker also:

```text
1. performs initial backfill, reconcile, prune, and vector-index checks before its loop
2. reconciles stale search rows every 60 seconds
3. retries ANN vector-index creation at most every 5 minutes
4. logs indexing rounds with batch and embedding counts
```

## Concurrency Controls

<MermaidDiagram
  caption="Concurrency boundaries: workers share AppState, search projection changes coordinate through a read/write lock, and model-heavy paths are rate-limited separately."
  :chart="concurrencyDiagram"
/>

### Projection Lock

Search rebuilds and index maintenance coordinate through a `RwLock`. Destructive projection resets,
normal search reads, and indexing writes share that lock.

### AI Model Request Concurrency

To keep concurrency bounded, the backend uses separate semaphores per lane:

1. summary/evaluator/chat/guardrails/planner share one lane (applies only to local models, cloud-tagged models skip that check)
2. search embedding/rerank/HyDE share another lane

**Currently, all semaphores are hard restricted at size 1.**

### Runtime Throttles

HTTP handlers share the request rate limiter from `AppState`. Cloud-model calls, YouTube quota
failures, and transcript dependency failures each have their own cooldown tracker.

## Client Processes

The product frontend and Tauri shell call the backend through the configured API base.
