# System Overview

## What dAstIll Is

dAstIll is a YouTube channel monitoring tool that helps you stop doom-scrolling and start deep-diving. It:

- **Monitors your channels**: Subscribe to YouTube channels, backfill their video history, and auto-refresh for new uploads
- **Extracts transcripts**: Pulls transcripts from videos so you can search and read instead of watch
- **Generates AI summaries**: Creates consistent, structured summaries using local or cloud LLMs via Ollama
- **Evaluates summary quality**: Uses a separate LLM-as-a-judge to score summaries against ground-truth transcripts
- **Enables search**: Full-text and optional semantic search across all transcripts and summaries
- **Preserves highlights**: Save and organize important snippets from transcripts and summaries

The goal is to help you quickly identify which videos are worth your time based on AI-generated insights and searchable content.

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
  - AWS S3 persistence for canonical data
  - AWS S3 Vectors for semantic search embeddings
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
- **AWS S3** for data storage
- **AWS S3 Vectors** for semantic search
- **AWS IAM** with GCP Workload Identity Federation for cross-cloud auth
- **Secret Manager** for YouTube API key

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

Local debug runs default semantic search on. Release / production builds default semantic search off unless explicitly enabled. The default embedding model is **embeddinggemma:latest** for generating vector embeddings via Ollama.
