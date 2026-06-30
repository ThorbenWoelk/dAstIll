# Local Development

## Prerequisites

Install:

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- [Ollama](https://ollama.com/) for local AI models

You also need local access to the backing services you plan to use:

- Google Application Default Credentials with access to the GCS data bucket
- a Google Cloud Storage bucket for app data
- optional Google Cloud Text-to-Speech access for summary audio
- a YouTube Data API key when you ingest YouTube sources
- an OpenAlex API key when you use authenticated OpenAlex search
- a local or private OpenAI-compatible ASR service when podcast feeds do not publish transcripts

For local GCP credentials, use:

```bash
gcloud auth application-default login
```

Clone the repo:

```bash
git clone https://github.com/ThorbenWoelk/dAstIll.git
cd dAstIll
```

## Start And Stop

The local stack consists of:

- a SvelteKit frontend on `3543` by default
- a Rust backend on `3544` by default
- a VitePress docs frontend on `4173` by default

From the repo root:

```bash
./start_app.sh
```

`./start_app.sh` stops any running dAstIll services first, then restarts the stack from a clean
state.

Detached mode:

```bash
./start_app.sh --detach
```

Detached startup writes supervisor output to `start_app.log` and service logs to `backend.log`,
`frontend.log`, and `docs.log`.

Stop everything cleanly:

```bash
./end_app.sh
```

Startup verifies the backend health endpoint and the initial workspace bootstrap response before it
reports success. If the bootstrap probe fails because local GCP credentials or `GCS_DATA_BUCKET` are
missing, startup stops and prints a hint about the credential source it found. Other bootstrap
failures also stop startup; check `backend.log`.

Process ownership, backend startup internals, worker loops, and shared runtime state are covered in
[Runtime Topology](/architecture/runtime-topology).

When you use `./start_app.sh`, it also augments local CORS config so the backend accepts both the
web frontend and the Tauri Android shell.

Default docs URL:

```text
http://localhost:4173
```

The app header includes a `Docs` link. In local development it falls back to
`http://localhost:4173`. Deployed frontend builds read the docs URL from frontend build config.

## Smoke Test Inputs

For a quick local ingest check, the workspace add-source input accepts:

- YouTube handles and channel URLs
- `openalex: <query>`
- `podcast: <feed-url>`
- `site: <page-url>` or a plain non-YouTube page URL

## Shared Env Files

The recommended local env layout is:

```text
~/.config/dastill/
  backend.env
  frontend.env
```

`./start_app.sh` and `./scripts/link_shared_env.sh` use this shared directory by default. Use the
repo-root env example files for the supported override keys.

Create or link the shared files from the repo root:

```bash
./scripts/link_shared_env.sh
```

What it does:

- migrates an existing worktree-local `backend/.env` or `frontend/.env` into the shared directory
  when the shared file does not exist yet
- creates `backend/.env` and `frontend/.env` symlinks that point at the shared files
- seeds missing shared files from `backend/.env.example` and `frontend/.env.example`

Env precedence for local development is:

1. shell environment variables
2. worktree-local `backend/.env` or `frontend/.env`
3. shared `~/.config/dastill/backend.env` or `~/.config/dastill/frontend.env`

Use the repo-root `backend/.env.example` and `frontend/.env.example` as the current variable lists.
Keep those example files as the place for exhaustive keys, defaults, and inline comments.

## Runtime Modes

`./start_app.sh` serves the live frontend by default.

There are two maintenance paths:

- `.github/runtime-mode.env` mirrors the release workflow. The script serves the
  maintenance/minimal frontend and keeps the backend running for `dastill-mini`.
- The local frontend-only preview mode skips backend startup and serves the maintenance frontend plus
  docs.

When either path enables maintenance mode, startup also exposes the mini reader at:

```text
http://localhost:3543/mini
```

For direct frontend-only commands, set the maintenance and support-link values in the frontend env.
Use `frontend/.env.example` for the current key names.

## Backend Env

Local backend startup reads the shared machine-local file at `~/.config/dastill/backend.env` by
default. If you want a one-off worktree override, `backend/.env` still works and wins over the shared
file. Shell environment variables win over both file-based sources.

Local startup needs `GCS_DATA_BUCKET` and valid Google Application Default Credentials for the GCS
provider. Set `GOOGLE_APPLICATION_CREDENTIALS` only when you need to force a specific service
account key file. If it is unset, the Google client libraries use the local ADC store created by
`gcloud auth application-default login`.

Smoke tests can opt into the in-memory object store with:

```env
OBJECT_STORE_PROVIDER=memory
ALLOW_IN_MEMORY_OBJECT_STORE=true
```

That provider is for tests and local process-only checks. It does not persist data across restarts.

Set optional tracing, operator access, and project-scoped API keys in `~/.config/dastill/backend.env`.
Use `backend/.env.example` for the current key names.

YouTube API keys are tied to the Google Cloud project that created them. If you change projects,
create a fresh key in the target project and update `~/.config/dastill/backend.env`. Production
secret rotation follows the production deployment flow.

## Local ASR

For local podcast transcription, enable local ASR in `~/.config/dastill/backend.env` and point it at
`localhost` or `127.0.0.1`. Use `backend/.env.example` for the current key names.

`./start_app.sh` then starts `./scripts/start_local_asr.sh --detach` before the backend. The helper
expects Homebrew `whisper-cpp` and `ffmpeg`, downloads `ggml-base.en.bin` into
`~/.cache/dastill/asr/`, and serves:

```text
http://127.0.0.1:5092/v1/audio/transcriptions
```

Stop it with `./end_app.sh` together with the rest of the local stack.

## Postman Debugging

The backend exposes a live OpenAPI document for local debugging.

When you run the full stack with `./start_app.sh`, import this URL into Postman:

```text
http://localhost:3544/api/openapi.json
```

If you run the backend binary by itself instead of `./start_app.sh`, the default port is `3001`:

```text
http://localhost:3001/api/openapi.json
```

Use the live OpenAPI URL as the source of truth during debugging. It reflects the running backend.
The checked-in `backend/openapi.postman.yaml` file is only a snapshot artifact and should not be
treated as the authoritative contract for local debugging.

## Android

The Android shell is covered in [Android Operations](/operations/mobile-tauri).
