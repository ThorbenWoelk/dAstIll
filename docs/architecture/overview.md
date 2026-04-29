---
aside: false
---
<script setup>
const systemContextDiagram = String.raw`
flowchart TB
  browser[Browser]
  backend[Backend<br/>Rust + Axum]
  app[Product UI<br/>SvelteKit]
  docs[Docs UI<br/>VitePress]
  sources[Content sources]
  ai[AI services]
  asr[STT]
  storage[Data stores]

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


# System Overview

## Primary Components

<MermaidDiagram
  caption="High-level system context."
  :chart="systemContextDiagram"
/>

### Product Frontend

- Built on **Svelte, SvelteKit**
- Serves mini UI + normal UI

### Backend

- Built with **Rust + Axum**
- API + background worker loops

### Custom STT

- local ASR (Automatic Speech Recognition) service
- Implements an OpenAI-compatible `POST /v1/audio/transcriptions` endpoint

### Docs Frontend

- Built with **VitePress** 

### Infrastructure

- **Firebase Hosting** for frontends
- **Cloud Run** for the backend and STT
- **AWS S3** for data storage
- **AWS S3 Vectors** for semantic search
- **local libSQL** for canonical video rows, user preferences, TTS statistics, and keyword search
- **Firebase** for auth and hosting
- **AWS IAM** with GCP Workload Identity Federation for cross-cloud auth
- **Secret Manager** for API keys and sensitive runtime config

### Deployment

- **Terraform** for IaC
- GitHub Actions for CI/CD

## Repo Structure

```text
dAstIll/
├── backend/     Rust + Axum API, workers, S3 storage, AI service adapters
├── frontend/    SvelteKit product UI
├── docs/        VitePress documentation frontend
└── terraform/   Cloud Run, secrets, and supporting infrastructure
```

## Data Flow

dAstIll is split into:

- **application** runtime processes
- **indexing** processes
- **background worker** processes

This keeps content flow, chunking and embedding out of normal CRUD operations.

## Core Design Rules

### Canonical before derived

Transcripts, summaries, and metadata live in canonical tables first. Search chunks and vector data are derived from those records and can be rebuilt.

### Async background processes

Transcript extraction, summary generation, summary evaluation, channel refreshes, and search projection maintenance are all driven by background loops.

### Local-first AI

The runtime supports local Ollama endpoints. Prod uses Ollama Cloud.
STT follows the same rule: use an operator-owned ASR endpoint for audio transcription if publisher transcripts are unavailable.
