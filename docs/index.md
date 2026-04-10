---
title: Documentation
pageClass: overview-page
---

# Documentation

Keep up with your content without the bad habits.
dAstIll tracks any source for you and provides distraction-free AI summaries so you can consciously decide on what's worth your time.

**Core capabilities:**

- **Source tracking**: Subscribe to YouTube channels, OpenAlex saved searches, podcast RSS feeds, and tracked website pages
- **AI summarization**: Generate consistent summaries evaluated by an LLM-as-judge for quality
- **Library chat**: Ask grounded questions across your saved library, with a per-message deep-research mode for wider synthesis
- **Highlights**: Save important snippets from transcripts and summaries for quick reference
- **Summary audio**: Generate spoken playback for ready summaries when Polly TTS is enabled
- **Hybrid search**: Full-text and semantic search across transcripts and summaries with context-aware chunking

**Current supported add-source inputs:**

- YouTube handles and channel URLs
- `openalex: <query>`
- `podcast: <feed-url>`
- `site: <page-url>` or a plain non-YouTube page URL

This site explains how the app works: the UI, the Rust backend, the AI and search pipeline, and the deployment setup.

- [System Overview](/architecture/overview) - major components, app/backend/docs boundaries, and repo structure.
- [UI Tour](/guides/ui-tour) - screenshots of the current mobile-web-first UI tour, with desktop notes where they matter.
- [Runtime Topology](/architecture/runtime-topology) - active processes, startup sequence, and shared runtime state.
- [Frontend and API](/architecture/frontend-and-api) - Svelte routes, startup flow, and handler boundaries.
- [Data Model](/architecture/data-model) - canonical tables, derived search projection, and status fields.
- [Search Indexing](/pipelines/search-indexing) - indexing worker phases and retrieval modes.
- [AI Models](/pipelines/ai-models) - model roles, fallback rules, and local versus production defaults.

## Operations

- [Local Development](/operations/local-development) - run the frontend, backend, and docs locally with the expected ports and startup flow.
- [Tauri Android](/operations/mobile-tauri) - local Android setup, `cargo tauri` commands, smoke tests, and APK output.
- [Deployment and Operations](/operations/deployment) - Cloud Run services, Terraform ownership, and CI/CD flow.
- [Content Flow](/pipelines/content-pipeline) - video ingestion, transcript extraction, summarization, evaluation, and indexing.

## Security

- [Security](/security/) - OWASP ASI mapping, current controls, and verification checklist.
- [OWASP ASI Status](/security/owasp-asi-status) - detailed status matrix of what is implemented and what remains open.
