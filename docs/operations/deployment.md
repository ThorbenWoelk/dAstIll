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
- `OLLAMA_EMBEDDING_MODEL`
- `SUMMARY_EVALUATOR_MODEL`
- `SUMMARIZE_PATH`
- log level

## CI/CD Flow

The GitHub Actions workflow:

```text
1. Applies Terraform to provision/update AWS and GCP resources
2. Builds and pushes backend image to Artifact Registry
3. Deploys backend to Cloud Run with AWS IAM role configuration
4. Builds and pushes the docs image
5. Deploys docs to Cloud Run as a public service
6. Builds the frontend image with the deployed backend URL injected as VITE_API_BASE
7. Deploys the frontend to Cloud Run with PUBLIC_DOCS_URL set to the deployed docs URL
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
