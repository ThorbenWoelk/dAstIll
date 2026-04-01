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
  fts[In-memory Tantivy BM25]
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

dAstIll is a YouTube channel monitoring tool that helps you stop doom-scrolling and start deep-diving. It:

- **Monitors your channels**: Subscribe to YouTube channels, backfill their video history, and auto-refresh for new uploads
- **Extracts transcripts**: Pulls transcripts from videos so you can search and read instead of watch
- **Generates AI summaries**: Creates consistent, structured summaries using local or cloud LLMs via Ollama
- **Evaluates summary quality**: Uses a separate LLM-as-a-judge to score summaries against ground-truth transcripts
- **Enables search**: Full-text and optional semantic search across all transcripts and summaries
- **Preserves highlights**: Save and organize important snippets from transcripts and summaries

## Primary Components

<MermaidDiagram
  caption="High-level system context: the product UI talks to the Rust backend, while the docs UI stays separate and static-first."
  :chart="systemContextDiagram"
/>

### Product Frontend

- Built with **SvelteKit** in `frontend/`
- Main workspace route at `/`
- Additional product routes:
  - `/download-queue`
  - `/highlights`
  - `/chat`
  - `/channels/[id]`
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
- Static-first and markdown-native

### Infrastructure

- **Terraform** in `terraform/`
- **Cloud Run** services for backend, product frontend, and docs frontend
- **AWS S3** for data storage
- **AWS S3 Vectors** for semantic search
- **Google Firestore** for video records, user preferences, and TTS statistics
- **AWS IAM** with GCP Workload Identity Federation for cross-cloud auth
- **Secret Manager** for API keys and sensitive runtime config (YouTube API key, Logfire token, Firebase client secrets)

## Repo-Level Boundaries

```text
frontend/  -> user-facing app interface
backend/   -> API, jobs, storage, AI orchestration
docs/      -> technical documentation frontend
terraform/ -> infrastructure state and service definitions
.specs/    -> persistent specs and task trackers
```

## Architectural Style

The application is intentionally split into:

- **canonical content storage** in regular application tables
- **derived search projection storage** for retrieval
- **background workers** that keep expensive or failure-prone work off user-facing writes

This avoids embedding, chunking, and external-model work directly inside normal CRUD operations.

<MermaidDiagram
  caption="Canonical content is stored first, then background workers build and maintain the derived search projection used by search and chat."
  :chart="canonicalFlowDiagram"
/>

## Core Design Rules

### Canonical before derived

Transcripts, summaries, and metadata live in canonical tables first. Search chunks and vector data are derived from those records and can be rebuilt.

### Async over inline

Transcript extraction, summary generation, summary evaluation, channel refreshes, and search projection maintenance are all driven by background loops.

### Local-first AI, cloud-capable evaluator path

The runtime supports local Ollama endpoints, cloud-backed model names, and explicit fallback policies. The app treats availability and rate limits as first-class runtime conditions.

### Semantic search is deployment-sensitive

Local debug runs default semantic search on. Release / production builds default semantic search off unless explicitly enabled. When semantic search is on, the backend reads the embedding model from `OLLAMA_EMBEDDING_MODEL`.
