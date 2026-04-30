<script setup>
const systemContextDiagram = String.raw`
flowchart TB
  browser[Browser]
  app[Product UI<br/>SvelteKit]
  docs[Docs UI<br/>VitePress]
  backend[Backend<br/>Rust + Axum]
  sources[Content sources]
  ai[AI services]
  asr[ASR service]
  storage[Storage]

  browser --> app
  browser --> docs
  app --> backend
  backend --> sources
  backend --> ai
  backend --> asr
  backend --> storage
`;

const canonicalFlowDiagram = String.raw`
flowchart TB
  source[Content source]
  canonical[Canonical records]
  workers[Background workers]
  projection[Search projection]
  retrieval[Workspace search + chat]

  source --> canonical
  canonical --> workers
  workers --> projection
  projection --> retrieval
`;
</script>

# System Overview

## System Shape

<MermaidDiagram caption="High-level system context." :chart="systemContextDiagram" />

| Area             | Owns                                                         | Detail doc                                          |
| ---------------- | ------------------------------------------------------------ | --------------------------------------------------- |
| Product frontend | Workspace UI, mini reader, auth entry points, client state   | [Frontend and API](/architecture/frontend-and-api)  |
| Backend          | HTTP API, durable writes, workers, AI/service adapters       | [Runtime Topology](/architecture/runtime-topology)  |
| Data model       | Canonical records, user-scoped records, derived projections  | [Data Model](/architecture/data-model)              |
| Content pipeline | Discovery, transcripts, summaries, evaluation, search sync   | [Content Pipeline](/pipelines/content-pipeline)     |
| Search           | Keyword index, semantic vectors, chunking, retrieval modes   | [Search Indexing](/pipelines/search-indexing)       |
| AI models        | Model roles, cooldowns, degradation                          | [AI Models](/pipelines/ai-models)                   |
| Chat             | RAG retrieval, streaming, attribution, budgets               | [Chat RAG](/pipelines/chat-rag)                     |
| Deployment       | Cloud Run, Firebase Hosting, Terraform, secrets, CI/CD       | [Deployment and Operations](/operations/deployment) |
| Mobile shell     | Tauri Android tooling, auth handoff, APK workflow, smoke set | [Tauri Android](/operations/mobile-tauri)           |

## Repository Map

```text
dAstIll/
├── backend/     Rust + Axum API, workers, storage, AI adapters
├── frontend/    SvelteKit product UI
├── docs/        VitePress documentation frontend
├── asr/         OpenAI-compatible podcast transcription service
└── terraform/   Cloud Run, Hosting, IAM, secrets, and billing infrastructure
```

## Core Flow

<MermaidDiagram
  caption="Canonical content is written first. Background workers derive search state later."
  :chart="canonicalFlowDiagram"
/>

```text
source input
  -> canonical records
  -> background workers
  -> search projection
  -> workspace search and chat
```

## Design Rules

### Canonical Before Derived

Transcripts, summaries, video metadata, and user-scoped state keep their own storage boundaries.
Search chunks, keyword indexes, vector embeddings, and generated audio are derived state.

Use this rule when deciding whether a write path should update source records or rebuild a
projection.

### Backend Owns Durable Writes

The backend owns durable writes and worker execution. Frontends call it as clients. The docs site has
no product runtime dependency.

Use this rule when deciding where a feature should persist state, run background work, or enforce
authorization.

### Workers Keep Heavy Work Off Request Paths

Transcript extraction, summary generation, summary evaluation, channel refreshes, gap scans, and
search indexing run through background workers.

Use this rule when adding slow work, model calls, or external API calls.

### AI Is Behind Service Boundaries

The backend talks to Ollama-compatible model endpoints and an OpenAI-compatible ASR endpoint. Model
selection and ASR hosting are runtime concerns, not frontend concerns.

Use this rule when adding summarization, evaluation, chat, embedding, reranking, TTS, or
transcription behavior.
