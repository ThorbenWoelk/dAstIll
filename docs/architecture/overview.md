# System Overview

## What dAstIll Is

dAstIll is a YouTube channel intelligence application. It tracks channels, ingests videos, extracts transcripts, generates summaries, evaluates summary quality, and indexes content for full-text or hybrid semantic search.

## Primary Components

### Product Frontend

- Built with **SvelteKit** in `frontend/`
- Main workspace route at `/`
- Additional product routes:
  - `/download-queue`
  - `/highlights`

### Backend

- Built with **Rust + Axum** in `backend/`
- Owns:
  - HTTP API
  - libSQL/Turso persistence
  - runtime config
  - AI service adapters
  - all long-running worker loops

### Documentation Frontend

- Built with **VitePress** in `docs/`
- Separate from the product UI
- Static-first and markdown-native

### Infrastructure

- **Terraform** in `terraform/`
- **Cloud Run** services for backend and product frontend
- **Secret Manager** for database credentials and YouTube API key

## Repo-Level Boundaries

```text
frontend/  -> user-facing app interface
backend/   -> API, jobs, storage, AI orchestration
docs/      -> technical documentation frontend
terraform/ -> infrastructure state and service definitions
```

## Architectural Style

The application is intentionally split into:

- **canonical content storage** in regular application tables
- **derived search projection storage** for retrieval
- **background workers** that keep expensive or failure-prone work off user-facing writes

This avoids embedding, chunking, and external-model work directly inside normal CRUD operations.

## Core Design Rules

### Canonical before derived

Transcripts, summaries, and metadata live in canonical tables first. Search chunks and vector data are derived from those records and can be rebuilt.

### Async over inline

Transcript extraction, summary generation, summary evaluation, channel refreshes, and search projection maintenance are all driven by background loops.

### Local-first AI, cloud-capable evaluator path

The runtime supports local Ollama endpoints, cloud-backed model names, and explicit fallback policies. The app treats availability and rate limits as first-class runtime conditions.

### Semantic search is deployment-sensitive

Local debug runs default semantic search on. Release / production builds default semantic search off unless explicitly enabled.
