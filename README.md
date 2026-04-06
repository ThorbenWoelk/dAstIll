# dAstIll

Stop doom-scrolling, start deep-diving. dAstIll monitors your favorite YouTube channels, pulls transcripts, and delivers AI-generated summaries - so you can quickly spot what matters to you and spend your time on the videos worth watching.

dAstIll is a full-stack Rust + SvelteKit application that uses Ollama LLMs to generate and quality-score summaries from transcripts.

## Features

- **Never miss a beat**: Track your favorite YouTube channels and filter what's worth watching without missing out.
- **Evaluated AI Summaries**: Dive deep without being overwhelmed. If an LLM screwed up, we will notice.
- **Highlights**: Mark and save important snippets from transcripts and summaries for quick reference.
- **Agentic RAG Search**: Ranked keyword and semantic search across transcripts and summaries, with timestamp metadata on supported transcript matches.
- **Chat with Content**: Ask questions across your video library with source attribution and multi-pass retrieval.
- **Vocabulary Customization**: Define word replacements applied during summary generation for consistent terminology.
- **Audio Playback**: Optional text-to-speech synthesis via Amazon Polly for listening to summaries.

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

## Tech Stack

### Frontend

- SvelteKit, TypeScript, bun

### Backend

Rust, AWS S3, AWS S3 Vectors, Turso (libSQL), Ollama

### Infrastructure & Deployment

Terraform, Google Cloud Run, AWS IAM (Workload Identity Federation), Google Secret Manager, Artifact Registry, GitHub Actions, Docker

## Prerequisites

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- [Ollama](https://ollama.com/) (required for local AI models)
- Turso database URL and auth token (`TURSO_DB_URL`, `TURSO_AUTH_TOKEN`)
- AWS credentials with access to S3 and S3 Vectors (via `~/.aws/credentials` or environment variables)
- An AWS S3 bucket for data storage and an S3 Vectors bucket for semantic search
- YouTube Data API Key (optional)

## Getting Started (Local Development)

1. **Clone the repository**:

   ```bash
   git clone https://github.com/ThorbenWoelk/dAstIll.git
   cd dAstIll
   ```

2. **Configure Environment Variables**:
   Set up the shared local env directory once per machine:

   ```bash
   ./scripts/link_shared_env.sh
   ```

   The default local workflow uses `~/.config/dastill/backend.env` and
   `~/.config/dastill/frontend.env`. The helper above migrates existing worktree-local
   `.env` files into that shared directory when possible and creates fresh symlinks for the
   current worktree.

   Backend env precedence is:
   - shell environment variables
   - `backend/.env` in the current worktree
   - `~/.config/dastill/backend.env`

   A typical backend config looks like this:

   ```env
   GCP_PROJECT_ID=your-gcp-project-id
   TURSO_DB_URL=libsql://your-turso-database.turso.io
   TURSO_AUTH_TOKEN=your-turso-auth-token
   AWS_REGION=eu-central-1
   S3_DATA_BUCKET=your-data-bucket
   S3_VECTOR_BUCKET=your-vectors-bucket
   S3_VECTOR_INDEX=search-chunks
   # Optional: custom endpoints (e.g. MinIO)
   # S3_ENDPOINT_URL=http://localhost:9000
   # S3_VECTOR_ENDPOINT_URL=http://localhost:9001
   # Optional: GCP AWS WIF path used in Cloud Run and some advanced local setups
   # AWS_ROLE_ARN="arn:aws:iam::877173393100:role/dastill-gcp-backend"
   # AWS_WIF_AUDIENCE="<backend-sa-unique-id>"
   BACKEND_PROXY_TOKEN=local-dev-backend-proxy-token
   BACKEND_CORS_ALLOWED_ORIGINS=http://localhost:3543
   YOUTUBE_API_KEY=optional-api-key
   OLLAMA_URL=http://localhost:11434
   OLLAMA_SUMMARY_MODEL=glm-5:cloud
   OLLAMA_DEFAULT_CHAT_MODEL=glm-5:cloud
   OLLAMA_FALLBACK_MODEL=qwen3-coder:30b
   SUMMARY_EVALUATOR_MODEL=qwen3.5:397b-cloud
   SEARCH_SEMANTIC_ENABLED=true
   OLLAMA_EMBEDDING_MODEL=embeddinggemma:latest
   SEARCH_AUTO_CREATE_VECTOR_INDEX=false
   SUMMARIZE_PATH=/opt/homebrew/bin/summarize
   ```

   `OLLAMA_SUMMARY_MODEL` and `SUMMARY_EVALUATOR_MODEL` must be different. If they match, backend startup exits before serving requests so summary evaluation stays independent from summary generation.
   If `OLLAMA_URL` points to a remote Ollama endpoint instead of localhost, also set `OLLAMA_API_KEY`.

   If you run the frontend separately from `start_app.sh`, keep its local values in
   `~/.config/dastill/frontend.env` and run `./scripts/link_shared_env.sh` in each
   worktree so direct frontend commands still see `frontend/.env`. Operator access is
   granted through the frontend server's `OPERATOR_EMAIL_ALLOWLIST`.

3. **Understand Search Defaults**:
   `SEARCH_SEMANTIC_ENABLED` overrides the runtime default:
   - Local debug builds default to semantic search on.
   - Release/production builds default to FTS-only unless `SEARCH_SEMANTIC_ENABLED=true` is explicitly set.
   - `SEARCH_SEMANTIC_ENABLED=false` disables embeddings in any environment.

   For local hybrid semantic search, configure `OLLAMA_EMBEDDING_MODEL` and leave `SEARCH_SEMANTIC_ENABLED` unset or set it to `true`.

4. **Start the Application**:
   You can start the frontend, backend, docs, and, when available, the Android shell using the provided startup script:

   ```bash
   ./start_app.sh
   ```

   `./start_app.sh` first shuts down any already-running dAstIll services, then starts the full stack again.

   To start the app in the background and return your shell immediately:

   ```bash
   ./start_app.sh --detach
   ```

   Detached mode starts a background supervisor, performs the usual health checks in the background, and writes its startup output to `start_app.log`. The service logs remain in `backend.log`, `frontend.log`, and `docs.log`.

   To stop everything cleanly:

   ```bash
   ./end_app.sh
   ```

5. **Sign-In And Roles Locally**:
   Anonymous browsing remains available by default. Signed-in users use the Firebase-backed `/login` flow, and operator-only actions depend on the frontend server's `OPERATOR_EMAIL_ALLOWLIST`.

## Tauri Android Development

dAstIll now includes a Tauri v2 shell for Android in [`src-tauri/`](./src-tauri). The Android app uses the same frontend bundle and talks directly to the Rust backend with Firebase bearer tokens.

Install the Tauri CLI once on your machine:

```bash
cargo install tauri-cli --version "^2"
```

If you do not want to install it globally, use `bunx @tauri-apps/cli@latest ...` instead of `cargo tauri ...`.

### Tooling

You need:

- Android Studio
- Java 17+
- Android SDK
- Android NDK
- Rust Android targets

Typical setup:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi \
  i686-linux-android x86_64-linux-android

export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
```

### Run On Android

If an Android emulator or device is connected, `./start_app.sh` launches the mobile shell automatically after the backend, frontend, and docs are ready.

```bash
./start_app.sh
```

To skip that auto-launch:

```bash
START_APP_SKIP_MOBILE=1 ./start_app.sh
```

If you want to run the shell manually instead:

```bash
cargo tauri android dev
```

### Build An APK

Debug APK:

```bash
cargo tauri android build -- --apk --debug
```

Release APK:

```bash
cargo tauri android build -- --apk
```

APK output:

```text
src-tauri/gen/android/app/build/outputs/apk/
```

### What To Verify

- The app launches and loads data from the backend.
- Anonymous mode works.
- Google sign-in works in the Android shell.
- Transcript text selection shows Android native `Highlight` and `Correct` actions.
- Highlight creation and vocabulary correction still work.
- Existing highlight deletion still works.

Detailed mobile steps live in [docs/local-development.md](./docs/local-development.md) and [docs/mobile-tauri.md](./docs/mobile-tauri.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
