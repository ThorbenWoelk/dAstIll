---
title: Project
---

# Project

## Repo Shape

This is a showcase repo with two working goals:

1. Explore technologies and architecture patterns such as Svelte, Rust, Firebase, and agentic RAG.
2. Push AI-assisted coding and product development in a real application.

Budget is tight. The codebase moves fast. Refactors, redesigns, and rough edges are part of the work. The steady goal is clearer structure, stronger security, clean code, and better operating habits.

Issues, criticism, and direct contributions are welcome.

## Features

- **Source library**: Track YouTube channels, OpenAlex saved searches, podcast feeds, and websites from one workspace.
- **Evaluated AI summaries**: Generate summaries and score them with a separate evaluator model so low-quality output can be detected.
- **Hybrid search**: Search transcripts, summaries, abstracts, notes, and page text with keyword and semantic retrieval.
- **Chat with content**: Ask grounded questions across the saved library with source attribution and optional multi-pass retrieval.
- **Highlights**: Save important snippets from transcripts and summaries for later review.
- **Vocabulary customization**: Define word replacements applied during summary generation for consistent terminology.
- **Summary audio**: Generate audio playback for summaries.
- **Mini reader**: Use `/mini` for an intentionally minimal reading surface.

## Supported Sources

dAstIll supports YouTube channels, OpenAlex saved searches, podcast feeds, and websites.

The exact add-source syntax lives in [Local Development](/operations/local-development).

## Stack

Frontend:

- SvelteKit
- TypeScript
- Bun

Backend:

- Rust
- Axum
- AWS S3
- AWS S3 Vectors
- local libSQL
- Ollama-compatible model endpoints
- Amazon Polly

Infrastructure:

- Terraform
- Firebase Hosting
- Google Cloud Run
- AWS IAM with Workload Identity Federation
- Google Secret Manager
- Artifact Registry
- GitHub Actions
- Docker

## Documentation Map

Start with the [documentation homepage](/) for the full map.
