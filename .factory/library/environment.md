# Environment

Environment variables, external dependencies, and setup notes.

**What belongs here:** Required env vars, external API keys/services, dependency quirks, platform-specific notes.
**What does NOT belong here:** Service ports/commands (use `.factory/services.yaml`).

---

## Deployment Target

- Google Cloud Run: 1 vCPU, 512Mi memory per service
- `cpu_idle = true` (scale to zero), `startup_cpu_boost = true`
- Frontend and Docs are public; Backend is private (only frontend SA can invoke)

## Critical Constraint

All performance optimizations MUST work within Cloud Run's 1 vCPU / 512Mi memory.
- Bounded concurrency for S3 parallelization (semaphore limit ~8-16)
- Memory-aware caching (bounded cache size, not unlimited)
- No optimization that assumes multi-core CPU availability

## AWS

- Region: eu-central-1
- S3 data bucket for storage
- S3 Vectors bucket with cosine similarity index (512-dim float32) for semantic search
- Cross-cloud auth: GCP-to-AWS Workload Identity Federation (OIDC) for production
- Local dev: dedicated IAM user with access keys via backend/.env

## Ollama

- Local: localhost:11434
- Production: cloud model endpoints
- Models: OLLAMA_MODEL and SUMMARY_EVALUATOR_MODEL must be different
- Embedding: embeddinggemma for semantic search

## Frontend

- Bun for package management and test runner
- SvelteKit with adapter-node
- Tailwind CSS v4
- Playwright for E2E tests (`bunx playwright test`; requires running app stack)

## Backend

- Rust 2024 edition
- Axum web framework with tokio runtime
- No ORM - direct S3 SDK calls
