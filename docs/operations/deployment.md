# Deployment and Operations

## Current Production Shape

The repository defines **three** Cloud Run services:

- backend
- product frontend
- docs frontend

## Infrastructure Ownership

Terraform manages:

- Cloud Run services (GCP)
- service accounts and IAM (GCP and AWS)
- AWS S3 bucket for data storage
- AWS S3 Vectors bucket and index for semantic search
- AWS IAM role for GCP Workload Identity Federation
- Secret Manager secrets (GCP)

## Cross-Cloud Authentication

The backend runs on Cloud Run but accesses AWS S3 and S3 Vectors. Authentication uses **GCP Workload Identity Federation**:

1. AWS IAM role (`backend_s3`) trusts GCP service account
2. Cloud Run backend receives `AWS_ROLE_ARN` and `AWS_WIF_AUDIENCE` env vars
3. Backend exchanges GCP identity token for AWS temporary credentials
4. All S3/S3 Vectors requests use the AWS credentials

Local development uses standard AWS credentials (`~/.aws/credentials` or environment).

## Secret and Config Boundaries

Secrets are stored in GCP Secret Manager for:

- `YOUTUBE_API_KEY`
- `LOGFIRE_TOKEN` (when Logfire observability is enabled for the backend)
- `firebase_web_api_key` and `firebase_auth_domain` (product frontend; Terraform writes secret versions from `terraform.tfvars`)

Non-secret runtime config is passed as plain env values for:

- `AWS_REGION`
- `S3_DATA_BUCKET`
- `S3_VECTOR_BUCKET`
- `S3_VECTOR_INDEX`
- `AWS_ROLE_ARN` (production only)
- `AWS_WIF_AUDIENCE` (production only)
- `OLLAMA_URL`
- `OLLAMA_MODEL`
- `OLLAMA_FALLBACK_MODEL`
- `OLLAMA_CHAT_MODEL`
- `OLLAMA_EMBEDDING_MODEL`
- `SUMMARY_EVALUATOR_MODEL`
- `SUMMARIZE_PATH`
- log level

### Firebase Auth (product frontend)

The SvelteKit app uses the Firebase JS SDK in the browser and **Firebase Admin** on the server for session cookies. The web client reads **`PUBLIC_FIREBASE_API_KEY`** (not `PUBLIC_FIREBASE_KEY`), **`PUBLIC_FIREBASE_AUTH_DOMAIN`**, and **`PUBLIC_FIREBASE_PROJECT_ID`** from `$env/dynamic/public`; the server resolves the same project for Admin SDK initialization.

**Terraform (`terraform.tfvars`, not GitHub Variables):** set `firebase_web_api_key` (Firebase console: Project settings > General > Web API Key) when you are ready to provision Firebase client secrets; leave unset or empty until then. Optionally set `firebase_auth_domain`; if omitted, Terraform stores `{project_id}.firebaseapp.com` in Secret Manager. Run `terraform apply` so secrets `dastill-firebase-web-api-key` and `dastill-firebase-auth-domain` exist and IAM allows the frontend Cloud Run service account and GitHub Actions deploy identity to read them.

**Release workflow:** mounts those secrets as `PUBLIC_FIREBASE_API_KEY` and `PUBLIC_FIREBASE_AUTH_DOMAIN`, and sets **`PUBLIC_FIREBASE_PROJECT_ID`** to the GCP project id (`GCP_PROJECT_ID` in the workflow), matching a Firebase project hosted in the same GCP project.

**GCP:** Terraform grants the frontend Cloud Run service account `roles/firebaseauth.admin` so the Node server can verify ID tokens and issue session cookies.

**Firebase console:** add your production site origin (and the Cloud Run URL if needed) under Authentication > Settings > Authorized domains.

## CI/CD Flow

The GitHub Actions workflow:

```text
1. Builds and pushes backend, docs, and frontend images to Artifact Registry
2. Deploys backend, docs, and frontend to Cloud Run (main branch or release dispatch)
3. Resolves deployed backend and docs URLs for the frontend service env
4. Deploys the frontend with runtime env including BACKEND_API_BASE, PUBLIC_DOCS_URL, PUBLIC_FIREBASE_PROJECT_ID, and Firebase client values from Secret Manager mounts
```

## Docker Layout

### Backend image

- built from `backend/Dockerfile`
- compiles Rust in a builder stage
- runs the `dastill` binary in a slim Debian runtime image
- bundles a `summarize` script path for transcript extraction

### Frontend image

- built from `frontend/Dockerfile`
- installs Bun during build
- generates the SvelteKit production output
- runs the Node adapter output at runtime

## Operational Notes

### Search in production

Production defaults to plain FTS mode unless `SEARCH_SEMANTIC_ENABLED=true` is intentionally set.

### Search vector index

ANN index creation is intentionally not part of startup migrations because it is too expensive for remote bulk indexing workflows.

### Docs frontend

The docs site is deployed as its own Cloud Run service from `docs/Dockerfile`. It serves the static VitePress build through nginx and remains operationally separate from the product frontend.

The `main`-branch deploy workflow publishes the docs revision with unauthenticated access enabled, so the service is reachable immediately after each successful deployment.

The product frontend links to this docs service through a `PUBLIC_DOCS_URL` runtime env var on the frontend Cloud Run service. Local development falls back to `http://localhost:4173` when that variable is unset.

Terraform grants the GitHub Actions deploy identity Cloud Run admin permissions and service-account-user bindings so the workflow can keep managing all three Cloud Run services.
