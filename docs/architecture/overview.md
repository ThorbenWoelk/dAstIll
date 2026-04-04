# System Overview

<script setup>
const systemContextDiagram = String.raw`
flowchart LR
  browser[Browser]

  subgraph surfaces["User-facing surfaces"]
    app[Product UI<br/>SvelteKit]
    docs[Docs UI<br/>VitePress]
  end

  backend[Backend<br/>Rust + Axum]

  subgraph storage["Durable storage"]
    s3[S3 data bucket<br/>channels, transcript/summary blobs, user-scoped records, search chunks]
    vectors[S3 Vectors<br/>semantic embeddings]
    firestore[Firestore<br/>videos, preferences, TTS stats]
  end

  subgraph external["External integrations"]
    youtube[YouTube APIs + subtitle fetch]
    ollama[Ollama models]
    polly[Amazon Polly]
  end

  browser --> app
  browser --> docs
  app --> backend
  backend --> s3
  backend --> vectors
  backend --> firestore
  backend --> youtube
  backend --> ollama
  backend --> polly
  docs --> browser
`;

const canonicalFlowDiagram = String.raw`
flowchart LR
  youtube[YouTube channel + video source]
  canonical[Canonical content<br/>channels, videos, transcripts, summaries, video_info]
  workers[Background workers]
  searchmeta[search_sources state]
  searchproj[search_chunks projection]
  fts[libSQL / Turso FTS5]
  vectors[S3 Vectors]
  retrieval[Workspace search + chat retrieval]

  youtube --> canonical
  canonical --> workers
  workers --> searchmeta
  searchmeta --> searchproj
  searchproj --> fts
  searchproj --> vectors
  fts --> retrieval
  vectors --> retrieval
`;
</script>

## What dAstIll Is

dAstIll is a source monitoring tool that is still migrating off an original YouTube-only model. It:

- **Monitors subscribed sources**: YouTube channels, OpenAlex saved searches, podcast RSS feeds, and tracked website pages
- **Extracts readable content**: Pulls transcripts, abstracts, show notes, or page text into the reading flow
- **Generates AI summaries**: Creates consistent, structured summaries using local or cloud LLMs via Ollama
- **Evaluates summary quality**: Uses a separate LLM-as-a-judge to score summaries against source text
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
  - `/download-queue`
  - `/highlights`
  - `/vocabulary`
  - `/chat`
  - `/login` and `/logout`

### Backend

- Built with **Rust + Axum** in `backend/`
- Owns:
  - HTTP API
  - AWS S3 persistence for canonical channel records, transcript/summary blobs, most user-scoped library records, and the search projection
  - AWS S3 Vectors for semantic search embeddings
  - Google Firestore for video records, user preferences, and TTS statistics
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
- **Google Firestore** for video records, user preferences, and TTS statistics
- **AWS IAM** with GCP Workload Identity Federation for cross-cloud auth
- **Secret Manager** for API keys and sensitive runtime config (YouTube API key, Logfire token, Firebase client secrets)

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
