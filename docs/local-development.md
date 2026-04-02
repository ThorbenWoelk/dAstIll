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

Startup now verifies both the backend health endpoint and the initial workspace bootstrap
response before it reports success. If local startup fails after the backend begins listening,
check `backend.log` for malformed Firestore video records or credential issues.

By default `./start_app.sh` forces the backend onto the local embedded libSQL search index even if
`~/.config/dastill/backend.env` contains Turso credentials. Set `START_APP_USE_TURSO=1` when you explicitly want
local startup to use the configured Turso replica path.

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

Local backend startup now reads the shared machine-local file at
`~/.config/dastill/backend.env` by default. If you want a one-off worktree override,
`backend/.env` still works and wins over the shared file. Shell environment variables
win over both file-based sources.

Typical flow:

```bash
./scripts/link_shared_env.sh
# edit ~/.config/dastill/backend.env
```

Important variables:

| Variable                            | Purpose                                                                                     |
| ----------------------------------- | ------------------------------------------------------------------------------------------- |
| `GCP_PROJECT_ID`                    | Google Cloud project id used for Firestore                                                  |
| `GOOGLE_APPLICATION_CREDENTIALS`    | Optional path to a local Firestore service-account JSON; falls back to ADC if unset/missing |
| `AWS_REGION`                        | AWS region for S3 and S3 Vectors                                                            |
| `S3_DATA_BUCKET`                    | S3 bucket for data storage                                                                  |
| `S3_VECTOR_BUCKET`                  | S3 Vectors bucket for semantic search                                                       |
| `S3_VECTOR_INDEX`                   | S3 Vectors index name for embeddings                                                        |
| `AWS_ACCESS_KEY_ID`                 | Local AWS access key used for S3 / S3 Vectors                                               |
| `AWS_SECRET_ACCESS_KEY`             | Local AWS secret key used for S3 / S3 Vectors                                               |
| `AWS_SESSION_TOKEN`                 | Optional temporary session token for local AWS auth                                         |
| `TURSO_DB_URL`                      | Optional Turso/libSQL database URL for durable keyword search                               |
| `TURSO_AUTH_TOKEN`                  | Turso auth token paired with `TURSO_DB_URL`                                                 |
| `BACKEND_PROXY_TOKEN`               | Shared secret used by the authenticated frontend proxy when it calls the backend            |
| `BACKEND_CORS_ALLOWED_ORIGINS`      | Comma-separated list of browser origins allowed to call the backend directly                |
| `AWS_ROLE_ARN` / `AWS_WIF_AUDIENCE` | Production only: GCP Workload Identity Federation for AWS                                   |
| `YOUTUBE_API_KEY`                   | Optional YouTube Data API access                                                            |
| `OLLAMA_URL`                        | Ollama endpoint                                                                             |
| `OLLAMA_API_KEY`                    | API key for Ollama cloud (required when using cloud Ollama URL)                             |
| `OLLAMA_SUMMARY_MODEL`              | Primary summarizer model                                                                    |
| `OLLAMA_FALLBACK_MODEL`             | Local fallback used when the primary summarizer is cloud-backed and rate-limited            |
| `OLLAMA_DEFAULT_CHAT_MODEL`         | Default chat model for RAG conversations (falls back to `OLLAMA_SUMMARY_MODEL` if not set)  |
| `SUMMARY_EVALUATOR_MODEL`           | Quality evaluator model - must differ from `OLLAMA_SUMMARY_MODEL`                           |
| `OLLAMA_EMBEDDING_MODEL`            | Search embedding model (required when semantic search is enabled)                           |
| `SEARCH_SEMANTIC_ENABLED`           | Explicit override for semantic search behavior                                              |
| `SEARCH_AUTO_CREATE_VECTOR_INDEX`   | Optional ANN index creation after backlog clears                                            |
| `SEARCH_RERANK_MODEL`               | Optional cross-encoder reranker model name (Ollama `/api/rerank`)                           |
| `SEARCH_HYDE_MODEL`                 | Optional HyDE generation model name (Ollama `/api/generate`, short queries only)            |
| `CHAT_MULTI_PASS_ENABLED`           | Enable multi-pass retrieval for chat (default: `true`)                                      |
| `DEFAULT_SEEDED_CHANNEL_ID`         | Fallback channel ID for empty workspace (default: set in config)                            |
| `BASELINE_RATE_LIMIT_PER_MINUTE`    | Baseline API rate limit per client (default: `600`)                                         |
| `EXPENSIVE_RATE_LIMIT_PER_MINUTE`   | Rate limit for AI/chat/search mutations (default: `120`)                                    |
| `ANONYMOUS_CHAT_QUOTA`              | Message quota for anonymous chat users (default: `30`)                                      |
| `SUMMARIZE_PATH`                    | Path to the transcript extraction CLI                                                       |
| `LOGFIRE_TOKEN`                     | Optional Logfire token for backend tracing / AI pipeline observability                      |
| `DATABRICKS_HOST`                   | Databricks workspace URL for analytics ingestion                                            |
| `DATABRICKS_TOKEN`                  | Databricks personal access token                                                            |
| `DATABRICKS_WAREHOUSE_ID`           | Databricks SQL warehouse ID                                                                 |
| `POLLY_TTS_ENABLED`                 | Enable Amazon Polly TTS for summary audio (default: `false`)                                |
| `POLLY_TTS_VOICE_ID`                | Polly voice ID (default: `Joanna`)                                                          |
| `POLLY_TTS_ENGINE`                  | Polly engine: `standard` or `neural` (default: `neural`)                                    |
| `POLLY_TTS_OUTPUT_FORMAT`           | Polly output format (default: `wav`)                                                        |
| `POLLY_TTS_SAMPLE_RATE`             | Polly sample rate in Hz (default: `16000`)                                                  |

The backend also needs Firestore credentials locally. Use one of these paths:

```bash
# Option 1: service-account JSON
GOOGLE_APPLICATION_CREDENTIALS=/absolute/path/to/service-account.json

# Option 2: application default credentials
gcloud auth application-default login
```

If `GOOGLE_APPLICATION_CREDENTIALS` points to a missing file, the backend removes that setting and falls back to application default credentials. If no valid Firestore credentials remain, startup fails before `http://localhost:3544/api/health` becomes ready.

The backend requires AWS credentials in addition to the bucket names. Provide them in
`~/.config/dastill/backend.env`:

```bash
AWS_ACCESS_KEY_ID=...
AWS_SECRET_ACCESS_KEY=...
# Optional for temporary credentials:
# AWS_SESSION_TOKEN=...
```

In production, Cloud Run uses `AWS_ROLE_ARN` and `AWS_WIF_AUDIENCE` for Workload Identity Federation instead of static access keys.

## Logfire Observability

The backend automatically switches to Logfire when `LOGFIRE_TOKEN` is present in
`~/.config/dastill/backend.env`.

Typical setup:

```bash
./scripts/link_shared_env.sh
# then uncomment LOGFIRE_TOKEN and paste your token
```

Behavior:

- with `LOGFIRE_TOKEN` set, backend `tracing` events are sent to Logfire
- without it, the backend keeps logging locally through `tracing_subscriber`
- current AI-related logs cover prompt lifecycle, retrieval timings, fallback/rate-limit events, and chat pipeline milestones
- raw prompt / generated-title preview logging is not enabled by default

## Frontend Auth And Proxy

The SvelteKit frontend proxies `/api/*` requests server-to-server using the backend proxy token. In Cloud Run it also mints an identity token for the backend audience, so the backend service remains non-public even though the product frontend is public.

Local defaults when you start with `./start_app.sh`:

| Variable              | Default                         |
| --------------------- | ------------------------------- |
| `BACKEND_PROXY_TOKEN` | `local-dev-backend-proxy-token` |

If you run the frontend by itself, keep its local values in
`~/.config/dastill/frontend.env`. The default shared workflow is to keep those values there
and run `./scripts/link_shared_env.sh` once per worktree so direct frontend commands
still see `frontend/.env`.

## Shared Env Directory

The recommended local env layout is:

```text
~/.config/dastill/
  backend.env
  frontend.env
```

Use the helper script from the repo root:

```bash
./scripts/link_shared_env.sh
```

What it does:

- migrates an existing worktree-local `backend/.env` or `frontend/.env` into the shared directory when the shared file does not exist yet
- creates `backend/.env` and `frontend/.env` symlinks that point at the shared files
- seeds missing shared files from `backend/.env.example` and `frontend/.env.example`

Env precedence for local development is:

1. shell environment variables
2. worktree-local `backend/.env` or `frontend/.env`
3. shared `~/.config/dastill/backend.env` or `~/.config/dastill/frontend.env`

Operator access is derived from `OPERATOR_EMAIL_ALLOWLIST` on the frontend server. Users whose Firebase email matches the allowlist receive the `operator` role in proxied backend requests.

### Auth Model

The current auth model is Firebase-based multi-user auth:

- Signed-in users receive a Firebase-backed session cookie handled by the SvelteKit server.
- Backend request identity is passed through `AccessContext` on proxied API calls.
- Persistent chat, channels, highlights, and preferences are authenticated user-scoped surfaces.
- Signed-out browsing remains available, but signed-out chat stays on the ephemeral path and is subject to the anonymous quota.
- Operator-only backend behavior is keyed off the proxied `operator` role, which comes from `OPERATOR_EMAIL_ALLOWLIST`.

## Search Defaults

`SEARCH_SEMANTIC_ENABLED` is an override, not the only switch:

- local debug runs default to semantic search on
- release builds default to plain FTS mode
- setting `SEARCH_SEMANTIC_ENABLED=false` disables embeddings even locally
- setting `SEARCH_SEMANTIC_ENABLED=true` enables embeddings in either environment

## Model Separation Guard

The backend refuses to start if `OLLAMA_SUMMARY_MODEL` and `SUMMARY_EVALUATOR_MODEL` are identical.

That check exists to keep summary generation and summary evaluation independent. If you copy the env template, keep the evaluator on a different model string than the summarizer.

## Recommended Working Loop

```text
1. Start frontend/backend/docs together with ./start_app.sh
2. Edit product code and docs side by side
3. Build the docs app before closing changes
```
