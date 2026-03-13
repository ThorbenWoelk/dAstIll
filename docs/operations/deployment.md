# Deployment and Operations

## Current Production Shape

The repository now defines **three** Cloud Run services:

- backend
- product frontend
- docs frontend

## Infrastructure Ownership

Terraform manages:

- Cloud Run services
- service accounts and IAM
- Secret Manager secrets
- Artifact Registry integration points

## Secret and Config Boundaries

Secrets are stored for:

- `DB_URL`
- `DB_PASS`
- `YOUTUBE_API_KEY`

Non-secret runtime config is passed as plain env values for:

- `OLLAMA_URL`
- `OLLAMA_MODEL`
- `SUMMARY_EVALUATOR_MODEL`
- `SUMMARIZE_PATH`
- log level

## CI/CD Flow

The GitHub Actions workflow:

```text
1. Builds and pushes backend image
2. Deploys backend to Cloud Run
3. Builds and pushes the docs image
4. Deploys docs to Cloud Run as a public service
5. Builds the frontend image with the deployed backend URL injected as VITE_API_BASE
6. Deploys the frontend to Cloud Run with PUBLIC_DOCS_URL set to the deployed docs URL
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
