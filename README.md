# dAstIll

Stop doom-scrolling, start deep-diving. dAstIll monitors your favorite YouTube channels, pulls transcripts, and delivers AI-generated summaries - so you can quickly spot what matters to you and spend your time on the videos worth watching.

daStIll is a full-stack Rust + SvelteKit application that uses local Ollama LLMs to generate and quality-score summaries from transcripts.
Built with a focus on reliability, performance, and clear architectural separation.

## Features

- **Channel Management**: Tracks favorite YouTube channels, backfills historical videos, and auto-refreshes for new content.
- **AI Summarization**: Generates insightful summaries in a consistent way, evaluated by LLM-as-a-judge for quality of summary related to ground-truth.
- **Highlights**: Mark and save important snippets from transcripts and summaries for quick reference.
- **Hybrid Search**: Full-text and optional semantic search across transcripts and summaries with highlighting.
- **Background Workers**: Automatic, asynchronous syncing, downloading, and generating of summaries and evals.

## Documentation

Detailed project documentation lives in the separate docs frontend under [`docs/index.md`](./docs/index.md).

- Docs landing page source: [`docs/index.md`](./docs/index.md)
- Architecture overview: [`docs/architecture/overview.md`](./docs/architecture/overview.md)
- Search indexing and retrieval: [`docs/search-indexing.md`](./docs/search-indexing.md)
- AI model behavior: [`docs/ai-models.md`](./docs/ai-models.md)

Run the docs frontend locally:

```bash
cd docs
bun install
bun run dev
```

Default local docs URL:

```text
http://localhost:4173
```

The app header includes a `Docs` link. In local development it falls back to `http://localhost:4173`; in deployed environments the frontend reads `PUBLIC_DOCS_URL` at runtime.

On `main` branch pushes, the docs site is also deployed through the same GitHub Actions workflow that deploys the backend and product frontend.

## Tech Stack

### Frontend

- SvelteKit, TypeScript, bun

### Backend

Rust, AWS S3, AWS S3 Vectors, Ollama

### Infrastructure & Deployment

Terraform, Google Cloud Run, AWS IAM (Workload Identity Federation), Google Secret Manager, Artifact Registry, GitHub Actions, Docker

## Prerequisites

To run the application locally, you will need:

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- [Ollama](https://ollama.com/) (running locally if using local AI models)
- AWS credentials with access to S3 and S3 Vectors (configured via `~/.aws/credentials` or environment variables)
- An AWS S3 bucket for data storage and an S3 Vectors bucket for semantic search
- (Optional) YouTube Data API Key.

## Getting Started (Local Development)

1. **Clone the repository**:

   ```bash
   git clone https://github.com/ThorbenWoelk/dAstIll.git
   cd dAstIll
   ```

2. **Configure Environment Variables**:
   Copy the backend template and fill in your local credentials:

   ```bash
   cp backend/.env.example backend/.env
   ```

   The backend reads `backend/.env` during local startup. A typical local config looks like this:

   ```env
   AWS_REGION=eu-central-1
   S3_DATA_BUCKET=your-data-bucket
   S3_VECTOR_BUCKET=your-vectors-bucket
   S3_VECTOR_INDEX=search-chunks
   YOUTUBE_API_KEY=optional-api-key
   OLLAMA_URL=http://localhost:11434
   OLLAMA_MODEL=glm-5:cloud
   OLLAMA_FALLBACK_MODEL=qwen3-coder:30b
   SUMMARY_EVALUATOR_MODEL=qwen3.5:397b-cloud
   SEARCH_SEMANTIC_ENABLED=true
   OLLAMA_EMBEDDING_MODEL=embeddinggemma
   SEARCH_AUTO_CREATE_VECTOR_INDEX=false
   SUMMARIZE_PATH=/opt/homebrew/bin/summarize
   ```

   `OLLAMA_MODEL` and `SUMMARY_EVALUATOR_MODEL` must be different. If they match, backend startup exits before serving requests so summary evaluation stays independent from summary generation.

3. **Understand Search Defaults**:
   `SEARCH_SEMANTIC_ENABLED` overrides the runtime default behavior:
   - Local debug runs (`cargo run`, `./start_app.sh`) default to semantic search on.
   - Release / production builds default to plain FTS mode unless you explicitly set `SEARCH_SEMANTIC_ENABLED=true`.
   - Setting `SEARCH_SEMANTIC_ENABLED=false` disables embeddings even locally.

   For local hybrid semantic search, keep `OLLAMA_EMBEDDING_MODEL` configured and either leave `SEARCH_SEMANTIC_ENABLED` unset or set it to `true`.

4. **Start the Application**:
   You can start the frontend, backend, and docs simultaneously using the provided startup script:

   ```bash
   ./start_app.sh
   ```

   To start the app in the background and return your shell immediately:

   ```bash
   ./start_app.sh --detach
   ```

   Detached mode starts a background supervisor, performs the usual health checks in the background, and writes its startup output to `start_app.log`. The service logs remain in `backend.log`, `frontend.log`, and `docs.log`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
