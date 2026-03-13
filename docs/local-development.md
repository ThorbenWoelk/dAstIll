# Local Development

## Product App

The product app consists of:

- a SvelteKit frontend on `3543` by default
- a Rust backend on `3544` by default

From the repo root:

```bash
./start_app.sh
```

Detached mode:

```bash
./start_app.sh --detach
```

Detached startup writes supervisor output to `start_app.log` and service logs to `backend.log` and `frontend.log`.

## Docs Frontend

The docs site is a separate frontend under `docs/`.

```bash
cd docs
bun install
bun run dev
```

Default docs URL:

```text
http://localhost:4173
```

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
| `DB_URL` / `DB_PASS`              | Turso/libSQL connection                                                          |
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
1. Start app frontend/backend with ./start_app.sh
2. Start docs frontend with cd docs && bun run dev
3. Edit product code and docs side by side
4. Build the docs app before closing changes
```
