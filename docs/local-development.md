# Local Development

## Product App

The product app consists of:

- a SvelteKit frontend on `3543` by default
- a Rust backend on `3544` by default
- a VitePress docs frontend on `4173` by default

From the repo root:

```bash
./start_app.sh
```

Detached mode:

```bash
./start_app.sh --detach
```

Detached startup writes supervisor output to `start_app.log` and service logs to `backend.log`, `frontend.log`, and `docs.log`.

Default docs URL:

```text
http://localhost:4173
```

## Docs Frontend

Build the static docs site:

```bash
cd docs
bun run build
```

The docs app also has a production container definition in `docs/Dockerfile`. Main-branch pushes build and deploy that image through the repository GitHub Actions workflow.

## Backend Environment

Local backend startup reads `backend/.env`.

Typical flow:

```bash
cp backend/.env.example backend/.env
```

Important variables:

| Variable                            | Purpose                                                                          |
| ----------------------------------- | -------------------------------------------------------------------------------- |
| `AWS_REGION`                        | AWS region for S3 and S3 Vectors                                                 |
| `S3_DATA_BUCKET`                    | S3 bucket for data storage                                                       |
| `S3_VECTOR_BUCKET`                  | S3 Vectors bucket for semantic search                                            |
| `S3_VECTOR_INDEX`                   | S3 Vectors index name for embeddings                                             |
| `AWS_ACCESS_KEY_ID`                 | Local AWS access key used for S3 / S3 Vectors                                    |
| `AWS_SECRET_ACCESS_KEY`             | Local AWS secret key used for S3 / S3 Vectors                                    |
| `AWS_SESSION_TOKEN`                 | Optional temporary session token for local AWS auth                              |
| `BACKEND_PROXY_TOKEN`               | Shared secret used by the authenticated frontend proxy when it calls the backend |
| `BACKEND_CORS_ALLOWED_ORIGINS`      | Comma-separated list of browser origins allowed to call the backend directly     |
| `AWS_ROLE_ARN` / `AWS_WIF_AUDIENCE` | Production only: GCP Workload Identity Federation for AWS                        |
| `YOUTUBE_API_KEY`                   | Optional YouTube Data API access                                                 |
| `OLLAMA_URL`                        | Ollama endpoint                                                                  |
| `OLLAMA_API_KEY`                    | API key for Ollama cloud (required when using cloud Ollama URL)                  |
| `OLLAMA_MODEL`                      | Primary summarizer model                                                         |
| `OLLAMA_FALLBACK_MODEL`             | Local fallback used when the primary summarizer is cloud-backed and rate-limited |
| `OLLAMA_CHAT_MODEL`                 | Chat model for RAG conversations (falls back to `OLLAMA_MODEL` if not set)       |
| `SUMMARY_EVALUATOR_MODEL`           | Quality evaluator model - must differ from `OLLAMA_MODEL`                        |
| `OLLAMA_EMBEDDING_MODEL`            | Search embedding model (default: embeddinggemma:latest)                          |
| `SEARCH_SEMANTIC_ENABLED`           | Explicit override for semantic search behavior                                   |
| `SEARCH_AUTO_CREATE_VECTOR_INDEX`   | Optional ANN index creation after backlog clears                                 |
| `SEARCH_RERANK_MODEL`               | Optional cross-encoder reranker model name (Ollama `/api/rerank`)                |
| `SEARCH_HYDE_MODEL`                 | Optional HyDE generation model name (Ollama `/api/generate`, short queries only) |
| `SUMMARIZE_PATH`                    | Path to the transcript extraction CLI                                            |
| `LOGFIRE_TOKEN`                     | Optional Logfire token for backend tracing / AI pipeline observability           |

For local development, the backend still needs AWS credentials in addition to the bucket names.
This repository now expects those local credentials in `backend/.env`:

```bash
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
# Optional for temporary credentials:
# AWS_SESSION_TOKEN=...
```

Production is different: Cloud Run uses `AWS_ROLE_ARN` and `AWS_WIF_AUDIENCE` for Workload Identity Federation, not static access keys.

## Logfire Observability

The backend automatically switches to Logfire when `LOGFIRE_TOKEN` is present in `backend/.env`.

Typical setup:

```bash
cp backend/.env.example backend/.env
# then uncomment LOGFIRE_TOKEN and paste your token
```

Behavior:

- with `LOGFIRE_TOKEN` set, backend `tracing` events are sent to Logfire
- without it, the backend keeps logging locally through `tracing_subscriber`
- current AI-related logs cover prompt lifecycle, retrieval timings, fallback/rate-limit events, and chat pipeline milestones
- raw prompt / generated-title preview logging is not enabled by default

## Frontend Auth And Proxy

The browser no longer talks to the backend directly. The SvelteKit frontend proxies `/api/*` requests server-to-server, treats anonymous visitors as regular users, and upgrades to operator access only after a password-based admin sign-in.

Local defaults when you start with `./start_app.sh`:

| Variable              | Default                         |
| --------------------- | ------------------------------- |
| `BACKEND_PROXY_TOKEN` | `local-dev-backend-proxy-token` |

If you run the frontend by itself, copy `frontend/.env.example` to `frontend/.env` and set `BACKEND_API_BASE`, `BACKEND_PROXY_TOKEN`, and `PUBLIC_DOCS_URL`. Admin sign-in uses `ADMIN_PASSWORD` from the runtime environment, and the minimal admin entrypoint is `/login`.

## Search Defaults

`SEARCH_SEMANTIC_ENABLED` is an override, not the only switch:

- local debug runs default to semantic search on
- release builds default to plain FTS mode
- setting `SEARCH_SEMANTIC_ENABLED=false` disables embeddings even locally
- setting `SEARCH_SEMANTIC_ENABLED=true` enables embeddings in either environment

## Model Separation Guard

The backend refuses to start if `OLLAMA_MODEL` and `SUMMARY_EVALUATOR_MODEL` are identical.

That check exists to keep summary generation and summary evaluation independent. If you copy the env template, keep the evaluator on a different model string than the summarizer.

## Recommended Working Loop

```text
1. Start frontend/backend/docs together with ./start_app.sh
2. Edit product code and docs side by side
3. Build the docs app before closing changes
```
