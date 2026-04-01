---
title: Documentation
pageClass: overview-page
---

# Documentation

dAstIll tracks your YouTube channels, fetches transcripts, and generates AI summaries so you can decide which videos are worth your time.

**Core capabilities:**

- **Channel tracking**: Subscribe to channels, backfill historical videos, and auto-refresh for new content
- **AI summarization**: Generate consistent summaries evaluated by an LLM-as-judge for quality
- **Highlights**: Save important snippets from transcripts and summaries for quick reference
- **Hybrid search**: Full-text and semantic search across transcripts and summaries with context-aware chunking

This site explains how the app works: the UI, the Rust backend, the AI and search pipeline, and the deployment setup.

- [System Overview](/architecture/overview) - major components, app/backend/docs boundaries, and repo structure.
- [UI Tour](/ui-tour) - screenshots of the current desktop and mobile UI.
- [Runtime Topology](/architecture/runtime-topology) - active processes, startup sequence, and shared runtime state.
- [Frontend and API](/architecture/frontend-and-api) - Svelte routes, startup flow, and handler boundaries.
- [Data Model](/architecture/data-model) - canonical tables, derived search projection, and status fields.
- [Search Indexing](/search-indexing) - indexing worker phases and retrieval modes.
- [AI Models](/ai-models) - model roles, fallback rules, and local versus production defaults.

## Operations

- [Local Development](/local-development) - run the frontend, backend, and docs locally with the expected ports and startup flow.
- [Deployment and Operations](/operations/deployment) - Cloud Run services, Terraform ownership, and CI/CD flow.
- [Content Flow](/flows/content-pipeline) - video ingestion, transcript extraction, summarization, evaluation, and indexing.

## Security

- [Security](/security/) - OWASP ASI mapping, current controls, and verification checklist.
- [OWASP ASI Status](/security/owasp-asi-status) - detailed status matrix of what is implemented and what remains open.
