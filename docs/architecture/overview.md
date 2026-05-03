<script setup>
const systemContextDiagram = String.raw`
flowchart TB
  browser[Browser]
  app[Frontend UI<br/>SvelteKit]
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

</script>

# System Overview

## System Shape

<MermaidDiagram caption="High-level system context." :chart="systemContextDiagram" />

| Area             | Owns                                                         | Detail doc                                          |
| ---------------- | ------------------------------------------------------------ | --------------------------------------------------- |
| Frontend         | Workspace UI, mini reader, auth entry points, client state   | [Frontend and API](/architecture/frontend-and-api)  |
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
├── frontend/    SvelteKit frontend
├── docs/        VitePress documentation frontend
├── asr/         OpenAI-compatible podcast transcription service
└── terraform/   Cloud Run, Hosting, IAM, secrets, and billing infrastructure
```

## Design Rules

### Canonical Before Derived

**Canonical**: Transcripts, summaries, metadata, and user-scoped state keep their own storage boundaries.

**Derived**: Search chunks, keyword indexes, vector embeddings, and generated audio are derived state.

### Headless frontends

The backend owns durable writes and worker execution. 
Frontends call it as clients.

### Async background workers

Transcript extraction, summary generation, summary evaluation, channel refreshes, gap scans, and
search indexing run through background workers.
