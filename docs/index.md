---
title: Documentation
pageClass: overview-page
---

# Documentation

dAstIll monitors your YouTube channels, pulls transcripts, and delivers AI-generated summaries - so you can quickly spot what matters and spend time on the videos worth watching.

**Core capabilities:**

- **Channel tracking**: Subscribe to channels, backfill historical videos, and auto-refresh for new content
- **AI summarization**: Generate consistent summaries evaluated by an LLM-as-judge for quality
- **Highlights**: Save important snippets from transcripts and summaries for quick reference
- **Hybrid search**: Full-text and semantic search across transcripts and summaries with context-aware chunking

This site is the technical reference for how the product is structured: the user-facing surfaces, the Rust backend, the AI and search pipeline, and the deployment boundaries.

<div class="overview-grid">
  <a class="overview-card" href="/local-development">
    <p class="overview-card-eyebrow">Get started</p>
    <h2>Local development</h2>
    <p>Run the frontend, backend, and docs locally with the expected ports and startup flow.</p>
  </a>
  <a class="overview-card" href="/architecture/overview">
    <p class="overview-card-eyebrow">Architecture</p>
    <h2>System overview</h2>
    <p>See the major components, repo layout, and the boundaries between product, backend, and infrastructure.</p>
  </a>
  <a class="overview-card" href="/flows/content-pipeline">
    <p class="overview-card-eyebrow">Pipeline</p>
    <h2>Content flow</h2>
    <p>Trace a video from channel discovery through transcript, summary, evaluation, and indexing.</p>
  </a>
  <a class="overview-card" href="/operations/deployment">
    <p class="overview-card-eyebrow">Operations</p>
    <h2>Deployment</h2>
    <p>Review the Cloud Run services, Terraform ownership, and the current config boundaries.</p>
  </a>
</div>

## What This Site Covers

### Product and Runtime

- [UI Tour](/ui-tour) - screenshots and route-level UX summary.
- [Runtime Topology](/architecture/runtime-topology) - active processes, startup sequence, and shared runtime state.
- [Frontend and API](/architecture/frontend-and-api) - Svelte routes, bootstrap flow, and handler boundaries.

### Data, AI, and Search

- [Data Model](/architecture/data-model) - canonical tables, derived search projection, and status fields.
- [Search Indexing](/search-indexing) - indexing worker phases and retrieval modes.
- [AI Models](/ai-models) - model roles, fallback policy, and local versus production defaults.

### Operations

- [Deployment and Operations](/operations/deployment) - Cloud Run services, Terraform ownership, and CI/CD flow.

## Repo Layout

```text
dAstIll/
├── backend/     Rust + Axum API, workers, S3 storage, AI service adapters
├── frontend/    SvelteKit product UI
├── docs/        VitePress documentation frontend
├── terraform/   Cloud Run, secrets, and supporting infrastructure
└── specs/       Persistent implementation specs and reference screenshots
```

## Scope

This site documents the application as it exists in this repository today. It complements the root README and does not imply a separate product surface beyond the docs frontend itself.
