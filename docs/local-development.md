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

| Variable                          | Purpose                                                                          |
| --------------------------------- | -------------------------------------------------------------------------------- |
| `AWS_REGION`                      | AWS region for S3 and S3 Vectors                                                 |
| `S3_DATA_BUCKET`                  | S3 bucket for data storage                                                       |
| `S3_VECTOR_BUCKET`                | S3 Vectors bucket for semantic search                                            |
| `S3_VECTOR_INDEX`                 | S3 Vectors index name for embeddings                                             |
| `AWS_ROLE_ARN` / `AWS_WIF_AUDIENCE` | Production only: GCP Workload Identity Federation for AWS                      |
| `YOUTUBE_API_KEY`                 | Optional YouTube Data API access                                                 |
| `OLLAMA_URL`                      | Ollama endpoint                                                                  |
| `OLLAMA_MODEL`                    | Primary summarizer model                                                         |
| `OLLAMA_FALLBACK_MODEL`           | Local fallback used when the primary summarizer is cloud-backed and rate-limited |
| `SUMMARY_EVALUATOR_MODEL`         | Quality evaluator model - must differ from `OLLAMA_MODEL`                        |
| `OLLAMA_EMBEDDING_MODEL`          | Search embedding model                                                           |
| `SEARCH_SEMANTIC_ENABLED`         | Explicit override for semantic search behavior                                   |
| `SEARCH_AUTO_CREATE_VECTOR_INDEX` | Optional ANN index creation after backlog clears                                 |
| `SUMMARIZE_PATH`                  | Path to the transcript extraction CLI                                            |

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
