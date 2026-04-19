---
aside: false
---

# System Overview

<script setup>
const systemContextDiagram = String.raw`
flowchart TB
  browser[Browser]
  backend[Backend<br/>Rust + Axum]
  app[Product UI<br/>SvelteKit]
  docs[Docs UI<br/>VitePress]
  sources[Content sources<br/>YouTube + OpenAlex + RSS + web]
  ai[AI services<br/>Ollama + Polly]
  storage[Data stores<br/>S3, S3 Vectors, local libSQL]

  browser --> app
  browser --> docs
  app --> backend
  backend --> sources
  backend --> ai
  backend --> storage
`;

const canonicalFlowDiagram = String.raw`
flowchart TB
  source[Content source]
  canonical[Canonical content]
  workers[Background workers]
  projection[Search projection]
  keyword[Keyword index<br/>local libSQL]
  vectors[Semantic index<br/>S3 Vectors]
  retrieval[Workspace search + chat]

  source --> canonical
  canonical --> workers
  workers --> projection
  projection --> keyword
  projection --> vectors
  keyword --> retrieval
  vectors --> retrieval
`;
</script>

## What dAstIll Is

dAstIll is a source monitoring tool that is still migrating off an original YouTube-only model. It:

- **Monitors subscribed sources**: YouTube channels, OpenAlex saved searches, podcast RSS feeds, and tracked website pages
- **Extracts readable content**: Pulls transcripts, abstracts, show notes, or page text into the reading flow
- **Generates AI summaries**: Creates consistent, structured summaries using local or cloud LLMs via Ollama
- **Evaluates summary quality**: Uses a separate LLM-as-a-judge to score summaries against source text
- **Supports library chat**: Lets users ask grounded questions across their saved content, with optional deep-research mode for wider synthesis
- **Synthesizes summary audio**: Can generate spoken playback for summaries through Amazon Polly when TTS is enabled
- **Enables search**: Full-text and optional semantic search across synced text content
- **Preserves highlights**: Save and organize important snippets from synced content

## Primary Components

<MermaidDiagram
  caption="High-level system context. The product UI talks to the Rust backend, while the docs UI stays separate."
  :chart="systemContextDiagram"
/>

### Product Frontend

- Built with **SvelteKit** in `frontend/`
- Main workspace route at `/`
- Additional product routes:
  - `/channels/[id]`
  - `/download-queue`
  - `/highlights`
  - `/mini`
  - `/vocabulary`
  - `/chat`
  - `/login` and `/logout`
- Browser builds register a service worker for static assets, API GET responses, and channel/video thumbnails; the registration is disabled in dev mode and in the Tauri runtime

### Backend

- Built with **Rust + Axum** in `backend/`
- Owns:
  - HTTP API
  - AWS S3 persistence for canonical channel records, transcript/summary blobs, most user-scoped library records, and the search projection
  - AWS S3 Vectors for semantic search embeddings
  - local libSQL storage for canonical video rows, user preferences, TTS statistics, and BM25 keyword search
  - Firebase/GCP identity and runtime config for auth, hosting, and project-aligned services
  - runtime config
  - AI service adapters
  - all long-running worker loops

### Documentation Frontend

- Built with **VitePress** in `docs/`
- Separate from the product UI
- Built from markdown and served as its own site

### Infrastructure

- **Terraform** in `terraform/`
- **Cloud Run** services for backend, product frontend, and docs frontend
- **AWS S3** for data storage
- **AWS S3 Vectors** for semantic search
- **local libSQL** for canonical video rows, user preferences, TTS statistics, and keyword search
- **Firebase** for auth and Hosting
- **AWS IAM** with GCP Workload Identity Federation for cross-cloud auth
- **Secret Manager** for API keys and sensitive runtime config (YouTube API key, OpenAlex API key, Logfire token, Firebase client secrets)

## Repo Layout

```text
dAstIll/
├── backend/     Rust + Axum API, workers, S3 storage, AI service adapters
├── frontend/    SvelteKit product UI
├── docs/        VitePress documentation frontend
├── terraform/   Cloud Run, secrets, and supporting infrastructure
└── .specs/      Persistent implementation specs and task trackers
```

## Architectural Style

The application is intentionally split into:

- **canonical content storage** in regular application tables
- **derived search projection storage** for retrieval
- **background workers** that keep expensive or failure-prone work off user-facing writes

This keeps embedding, chunking, and external model calls out of normal CRUD operations.

<MermaidDiagram
  caption="Canonical content is stored first, then background workers build and maintain the derived search projection used by search and chat."
  :chart="canonicalFlowDiagram"
/>

## Core Design Rules

### Canonical before derived

Transcripts, summaries, and metadata live in canonical tables first. Search chunks and vector data are derived from those records and can be rebuilt.

### Async over inline

Transcript extraction, summary generation, summary evaluation, channel refreshes, and search projection maintenance are all driven by background loops.

### Local-first AI, cloud-backed evaluator support

The runtime supports local Ollama endpoints, cloud-backed model names, and explicit fallback rules. The app treats availability and rate limits as normal runtime conditions that must be handled.

### Semantic search defaults depend on the environment

Local debug runs default semantic search on. Release / production builds default semantic search off unless explicitly enabled. When semantic search is on, the backend reads the embedding model from `OLLAMA_EMBEDDING_MODEL`.
