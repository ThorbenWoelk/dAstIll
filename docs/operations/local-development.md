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

`./start_app.sh` always stops any running dAstIll services first, then restarts the stack from a clean state.

Detached mode:

```bash
./start_app.sh --detach
```

Detached startup writes supervisor output to `start_app.log` and service logs to `backend.log`, `frontend.log`, and `docs.log`.

Stop everything cleanly:

```bash
./end_app.sh
```

Startup verifies both the backend health endpoint and the initial workspace bootstrap
response before it reports success. If the bootstrap probe fails because local AWS credentials are
missing, expired, or still pinned to a temporary session in `backend.env`, startup stops and prints
an explicit hint about the credential source it found. Other bootstrap failures also stop startup;
check `backend.log`.

When you use `./start_app.sh`, it also augments `BACKEND_CORS_ALLOWED_ORIGINS` for local runtime
so the backend accepts both the web frontend and the Tauri Android shell (`http://tauri.localhost`)
even if your shared env file only lists the browser origin.

The workspace add-source input currently accepts:

- YouTube handles and channel URLs
- `openalex: <query>`
- `podcast: <feed-url>`
- `site: <page-url>` or a plain non-YouTube page URL

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

The docs app is deployed from the static VitePress build through Firebase Hosting. Main-branch pushes build the site and publish `docs/.vitepress/dist` through the repository GitHub Actions workflow.

## Tauri Android Development

The repo now includes a Tauri v2 shell in `src-tauri/`.

Install the CLI once:

```bash
cargo install tauri-cli --version "^2"
```

If `cargo tauri` is not installed, use `bunx` instead:

```bash
bunx @tauri-apps/cli@latest dev
bunx @tauri-apps/cli@latest android dev
```

Typical local setup:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi \
  i686-linux-android x86_64-linux-android

export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
```

Check the Android device list:

```bash
adb devices
```

Recommended run loop:

```bash
./start_app.sh
```

When an Android emulator or device is connected, `./start_app.sh` starts the Tauri Android shell automatically after the local services are healthy.

To skip the mobile shell:

```bash
START_APP_SKIP_MOBILE=1 ./start_app.sh
```

To run the shell manually:

```bash
cargo tauri android dev
```

Build APKs:

```bash
cargo tauri android build -- --apk --debug
cargo tauri android build -- --apk
```

APK output:

```text
src-tauri/gen/android/app/build/outputs/apk/
```

Android-specific smoke checks:

- app launches without a blank screen
- anonymous mode works
- backend data loads
- Google sign-in works
- transcript text selection shows native `Highlight` and `Correct` actions
- highlight creation, correction flow, and highlight deletion still work

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

| Variable                            | Purpose                                                                                      |
| ----------------------------------- | -------------------------------------------------------------------------------------------- |
| `GCP_PROJECT_ID`                    | Google Cloud project id used for Firestore                                                   |
| `GOOGLE_APPLICATION_CREDENTIALS`    | Optional path to a local Firestore service-account JSON; falls back to ADC if unset/missing  |
| `AWS_REGION`                        | AWS region for S3 and S3 Vectors                                                             |
| `S3_DATA_BUCKET`                    | S3 bucket for data storage                                                                   |
| `S3_VECTOR_BUCKET`                  | S3 Vectors bucket for semantic search                                                        |
| `S3_VECTOR_INDEX`                   | S3 Vectors index name for embeddings                                                         |
| `AWS_SHARED_CREDENTIALS_FILE`       | Optional override for the shared AWS credentials file used by the local SDK default chain     |
| `AWS_CONFIG_FILE`                   | Optional override for the shared AWS config file (region/profile metadata)                    |
| `AWS_ACCESS_KEY_ID`                 | Fallback inline AWS access key used for S3 / S3 Vectors; avoid for routine local development  |
| `AWS_SECRET_ACCESS_KEY`             | Fallback inline AWS secret key paired with `AWS_ACCESS_KEY_ID`                                |
| `AWS_SESSION_TOKEN`                 | Temporary session token only; do not keep this set for permanent local development            |
| `TURSO_DB_URL`                      | Optional Turso/libSQL database URL for durable keyword search                                |
| `TURSO_AUTH_TOKEN`                  | Turso auth token paired with `TURSO_DB_URL`                                                  |
| `BACKEND_PROXY_TOKEN`               | Shared secret for trusted first-party callers that use the backend's proxy-auth header path  |
| `BACKEND_CORS_ALLOWED_ORIGINS`      | Comma-separated list of browser origins allowed to call the backend directly                 |
| `AWS_ROLE_ARN` / `AWS_WIF_AUDIENCE` | Production only: GCP Workload Identity Federation for AWS                                    |
| `YOUTUBE_API_KEY`                   | Optional YouTube Data API access; project-scoped, so rotate it when `GCP_PROJECT_ID` changes |
| `OLLAMA_URL`                        | Ollama endpoint                                                                              |
| `OLLAMA_API_KEY`                    | API key for Ollama cloud (required when using cloud Ollama URL)                              |
| `OLLAMA_SUMMARY_MODEL`              | Primary summarizer model                                                                     |
| `OLLAMA_FALLBACK_MODEL`             | Local fallback used when the primary summarizer is cloud-backed and rate-limited             |
| `OLLAMA_DEFAULT_CHAT_MODEL`         | Default chat model for RAG conversations (falls back to `OLLAMA_SUMMARY_MODEL` if not set)   |
| `SUMMARY_EVALUATOR_MODEL`           | Quality evaluator model - must differ from `OLLAMA_SUMMARY_MODEL`                            |
| `OLLAMA_EMBEDDING_MODEL`            | Search embedding model (required when semantic search is enabled)                            |
| `SEARCH_SEMANTIC_ENABLED`           | Explicit override for semantic search behavior                                               |
| `SEARCH_AUTO_CREATE_VECTOR_INDEX`   | Optional ANN index creation after backlog clears                                             |
| `SEARCH_RERANK_MODEL`               | Optional cross-encoder reranker model name (Ollama `/api/rerank`)                            |
| `SEARCH_HYDE_MODEL`                 | Optional HyDE generation model name (Ollama `/api/generate`, short queries only)             |
| `CHAT_MULTI_PASS_ENABLED`           | Enable multi-pass retrieval for chat (default: `true`)                                       |
| `DEFAULT_SEEDED_CHANNEL_ID`         | Fallback channel ID for empty workspace (default: set in config)                             |
| `BASELINE_RATE_LIMIT_PER_MINUTE`    | Baseline API rate limit per client (default: `600`)                                          |
| `EXPENSIVE_RATE_LIMIT_PER_MINUTE`   | Rate limit for AI/chat/search mutations (default: `120`)                                     |
| `ANONYMOUS_CHAT_QUOTA`              | Message quota for anonymous chat users (default: `30`)                                       |
| `SUMMARIZE_PATH`                    | Path to the transcript extraction CLI                                                        |
| `LOGFIRE_TOKEN`                     | Optional Logfire token for backend tracing / AI pipeline observability                       |
| `DATABRICKS_HOST`                   | Databricks workspace URL for analytics ingestion                                             |
| `DATABRICKS_TOKEN`                  | Databricks personal access token                                                             |
| `DATABRICKS_WAREHOUSE_ID`           | Databricks SQL warehouse ID                                                                  |
| `POLLY_TTS_ENABLED`                 | Enable Amazon Polly TTS for summary audio (default: `false`)                                 |
| `POLLY_TTS_VOICE_ID`                | Polly voice ID (default: `Joanna`)                                                           |
| `POLLY_TTS_ENGINE`                  | Polly engine: `standard` or `neural` (default: `neural`)                                     |
| `POLLY_TTS_OUTPUT_FORMAT`           | Polly output format (default: `wav`)                                                         |
| `POLLY_TTS_SAMPLE_RATE`             | Polly sample rate in Hz (default: `16000`)                                                   |

The backend also needs Firestore credentials locally. Use one of these paths:

```bash
# Option 1: service-account JSON
GOOGLE_APPLICATION_CREDENTIALS=/absolute/path/to/service-account.json

# Option 2: application default credentials
gcloud auth application-default login
```

If `GOOGLE_APPLICATION_CREDENTIALS` points to a missing file, the backend removes that setting and falls back to application default credentials. If no valid Firestore credentials remain, startup fails before `http://localhost:3544/api/health` becomes ready.

The preferred local AWS setup is a shared credentials file outside the repo-owned `.env` files.
Create these machine-local files:

```bash
mkdir -p ~/.config/dastill/aws
cat > ~/.config/dastill/aws/credentials <<'EOF'
[default]
aws_access_key_id = YOUR_LONG_LIVED_ACCESS_KEY
aws_secret_access_key = YOUR_LONG_LIVED_SECRET_KEY
EOF

cat > ~/.config/dastill/aws/config <<'EOF'
[default]
region = eu-central-1
EOF
```

The backend auto-detects `~/.config/dastill/aws/credentials` and `~/.config/dastill/aws/config`
when they exist. You only need `AWS_SHARED_CREDENTIALS_FILE` / `AWS_CONFIG_FILE` in
`~/.config/dastill/backend.env` if you want to override those default paths.

Inline `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` in `~/.config/dastill/backend.env` are still
supported as a fallback, but they override the shared credentials file. Remove any old
`AWS_SESSION_TOKEN` line if you want permanent local credentials; leaving a stale session token in
`backend.env` forces the backend onto temporary STS credentials even when the AWS CLI can log in.

To migrate an existing permanent inline keypair out of `backend.env`:

```bash
./scripts/migrate_local_aws_credentials.sh
```

That helper intentionally refuses to migrate temporary `ASIA...` / `AWS_SESSION_TOKEN` credentials.

If you only have temporary SSO-backed credentials available, log in with the profile you want and
sync the exported keypair into `~/.config/dastill/backend.env`:

```bash
aws sso login --profile your-profile
./scripts/sync_aws_programmatic_credentials.sh your-profile
```

That path is useful for short-lived sessions, but the shared credentials file remains the preferred
permanent local setup.

In production, Cloud Run uses `AWS_ROLE_ARN` and `AWS_WIF_AUDIENCE` for Workload Identity Federation instead of static access keys.

`YOUTUBE_API_KEY` is tied to the Google Cloud project that created it. If you migrate from one GCP project to another, create a fresh key in the target project, update `~/.config/dastill/backend.env`, and keep that value aligned with `terraform/terraform.tfvars` so local and production validation behave the same way.

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

## Frontend Runtime

The frontend now builds as a static bundle. Browser and Tauri clients call the Rust backend directly using `VITE_API_BASE`, and authenticated requests send the Firebase ID token as `Authorization: Bearer <token>`.

In production the static frontend and docs are served by Firebase Hosting. Local development still uses the Vite dev server for the app and VitePress for docs.

Optional browser-auth override for the Android system-browser sign-in handoff:

- `PUBLIC_BROWSER_AUTH_BASE_URL` or `VITE_BROWSER_AUTH_BASE_URL` forces the browser origin used when the Tauri Android shell opens the external `/login` flow.

If you run the frontend by itself, keep its local values in
`~/.config/dastill/frontend.env`. The default shared workflow is to keep those values there
and run `./scripts/link_shared_env.sh` once per worktree so direct frontend commands
still see `frontend/.env`.

The Tauri Android dev shell uses the same shared/local frontend env files. When `VITE_API_BASE` is unset and the app is running from `http://tauri.localhost`, the frontend falls back to `http://127.0.0.1:3544`, which matches the `adb reverse` mapping created by `./start_app.sh`.

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

Operator access is derived from `OPERATOR_EMAIL_ALLOWLIST` on the backend. Users whose Firebase email matches the allowlist receive the `operator` role when the backend validates their bearer token.

### Auth Model

The current auth model is Firebase-based multi-user auth:

- Signed-in users keep their Firebase browser/session state client-side.
- Backend request identity is derived either from trusted first-party proxy headers or from direct Firebase bearer-token validation.
- Persistent chat, channels, highlights, and preferences are authenticated user-scoped surfaces.
- Signed-out browsing remains available, but signed-out chat stays on the ephemeral path and is subject to the anonymous quota.
- Operator-only backend behavior is keyed off the validated Firebase email allowlist.

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
