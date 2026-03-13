---
layout: home

hero:
  name: "dAstIll"
  text: "System Documentation"
  tagline: "Architecture, workers, AI model behavior, search indexing, data flows, and operations."
  actions:
    - theme: brand
      text: Read Architecture
      link: /architecture/overview
    - theme: alt
      text: Local Development
      link: /local-development

features:
  - title: Runtime Topology
    details: Understand the three user-facing frontends, the Rust backend, and the five always-on worker loops that keep the system progressing.
  - title: AI and Search Behavior
    details: See how summarization, evaluation, transcript cleanup, and hybrid search are configured, gated, and degraded across local and production runtimes.
  - title: Data and Flow Documentation
    details: Trace how channels, videos, transcripts, summaries, highlights, and search chunks move from ingestion through indexing and retrieval.
---

## Why This Site Exists

The repository README is intentionally short. This site is the longer-lived technical reference for how dAstIll is wired today.

It is a separate frontend built with **VitePress** because that keeps the docs:

- static-first
- markdown-native
- low-maintenance
- easy to build with Bun
- separate from the product UI and deployment runtime

## Documentation Map

- [System Overview](/architecture/overview) - major components and repo layout
- [Runtime Topology](/architecture/runtime-topology) - process model, parallel workers, and startup behavior
- [Frontend and API](/architecture/frontend-and-api) - Svelte routes, bootstrap flow, and handler boundaries
- [Data Model](/architecture/data-model) - canonical tables, search projection tables, and status fields
- [Content Pipeline](/flows/content-pipeline) - ingestion through transcript, summary, evaluation, and search hooks
- [Search Indexing](/search-indexing) - indexing worker phases and retrieval modes
- [AI Models](/ai-models) - model roles, fallback policy, cooldowns, and local-vs-prod defaults
- [Deployment](/operations/deployment) - Cloud Run, Terraform, Docker, and current hosting boundaries
- [UI Tour](/ui-tour) - screenshots and route-level UX summary

## Repo Layout

```text
dAstIll/
├── backend/     Rust + Axum API, workers, libSQL/Turso access, AI service adapters
├── frontend/    SvelteKit product UI
├── docs/        VitePress documentation frontend
├── terraform/   Cloud Run, secrets, and supporting infrastructure
└── .specs/      Persistent implementation specs and task tracking
```

## Current Scope

This docs site documents the application as it exists in this repository today. It does **not** currently imply a production deployment for the docs site itself.
