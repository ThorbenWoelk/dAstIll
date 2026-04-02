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
- optional BigQuery billing export dataset prerequisites
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

- `OLLAMA_API_KEY`
- `YOUTUBE_API_KEY`
- `LOGFIRE_TOKEN` (when Logfire observability is enabled for the backend)
- `BACKEND_PROXY_TOKEN`
- `TURSO_AUTH_TOKEN` (when Turso-backed keyword search is enabled in production)
- `DATABRICKS_TOKEN` (only when Databricks ingestion is configured)
- `firebase_web_api_key` and `firebase_auth_domain` (product frontend; Terraform derives both from the Firebase web app config and writes them to Secret Manager)

Non-secret backend runtime config is passed as plain env values for:

- `AWS_REGION`
- `S3_DATA_BUCKET`
- `S3_VECTOR_BUCKET`
- `S3_VECTOR_INDEX`
- `TURSO_DB_URL` (when Turso-backed keyword search is enabled in production)
- `AWS_ROLE_ARN` (production only)
- `AWS_WIF_AUDIENCE` (production only)
- `OLLAMA_URL`
- `OLLAMA_SUMMARY_MODEL`
- `OLLAMA_FALLBACK_MODEL`
- `OLLAMA_DEFAULT_CHAT_MODEL`
- `OLLAMA_EMBEDDING_MODEL`
- `SUMMARY_EVALUATOR_MODEL`
- `SUMMARIZE_PATH`
- log level

Non-secret product frontend runtime config is passed as plain env values for:

- `BACKEND_API_BASE`
- `BACKEND_IDENTITY_AUDIENCE`
- `PUBLIC_DOCS_URL`
- `PUBLIC_FIREBASE_PROJECT_ID`
- `PUBLIC_CONTACT_EMAIL`

### Firebase Auth (product frontend)

The SvelteKit app uses the Firebase JS SDK in the browser and **Firebase Admin** on the server for session cookies. The web client reads the Firebase Web API key as **`PUBLIC_FIREBASE_API_KEY`** (alias **`PUBLIC_FIREBASE_KEY`**), plus **`PUBLIC_FIREBASE_AUTH_DOMAIN`** and **`PUBLIC_FIREBASE_PROJECT_ID`**, from `$env/dynamic/public`; the server resolves the same project for Admin SDK initialization.

**Terraform (`terraform.tfvars`, not GitHub Variables):** Terraform creates the Firebase project resources and web app, then reads the effective Web API key and auth domain from the Firebase web app config data source before writing them to Secret Manager. Run `terraform apply` so secrets `dastill-firebase-web-api-key` and `dastill-firebase-auth-domain` exist and IAM allows the frontend Cloud Run service account to read them.

**Google sign-in:** anonymous auth stays enabled through Identity Platform. Google sign-in itself is managed through [`frontend/firebase.json`](../../frontend/firebase.json) and should be deployed separately with `bunx firebase-tools@15.12.0 deploy --only auth --project "$PROJECT_ID" --config frontend/firebase.json --non-interactive` when provisioning a new project or when that file changes. That lets Firebase provision the correct project-local Google OAuth client for the web app instead of reusing a copied client ID/secret from another project.

**Release workflow:** uses the Terraform-managed frontend Firebase secrets `dastill-firebase-web-api-key` and `dastill-firebase-auth-domain`, mounting them as `PUBLIC_FIREBASE_API_KEY` and `PUBLIC_FIREBASE_AUTH_DOMAIN`. It sets **`PUBLIC_FIREBASE_PROJECT_ID`** to the GCP project id (`GCP_PROJECT_ID` in the workflow), plus frontend runtime env such as `BACKEND_API_BASE`, `BACKEND_IDENTITY_AUDIENCE`, `PUBLIC_DOCS_URL`, and `PUBLIC_CONTACT_EMAIL`. Routine releases do not redeploy Firebase Auth config.

**GCP:** Terraform grants `roles/firebaseauth.admin` to the frontend Cloud Run service account so the Node server can verify ID tokens and issue session cookies.

**Authorized domains:** Terraform manages Identity Platform authorized domains. The default set includes `localhost`, the Firebase-hosted domains for the project, the deployed frontend Cloud Run host, and any entries in `firebase_authorized_domains_extra`. Use Terraform rather than console-only edits for managed environments.

## Project Migration

To cut over from a previous GCP project to the current `dastill` project:

1. Create or gain access to the `dastill` GCP project, attach billing, and decide the Firestore location before the first apply. The repo now exposes `firestore_location_id` explicitly; the example uses `eur3`.
2. Update your local `terraform.tfvars` for the new target project. Set `project_id = "dastill"` and keep `app_name = "dastill"` unless you intentionally want new GCP/AWS resource names. If the GitHub Workload Identity Pool lives outside the target project, also set `github_wif_pool_project_number`; otherwise it defaults to the active project number.
3. Decide how Terraform state will handle the shared AWS resources. Buckets, vector buckets, and the `dastill-gcp-backend` AWS role are keyed by `app_name`, not `project_id`. Reusing the existing state is the simplest cutover. If you start from a fresh state backend, import the existing AWS resources before apply or intentionally rename `app_name` and migrate that data separately.
4. Apply Terraform against the new project and record the outputs you need for GitHub. At minimum, update repository secrets `GCP_PROJECT_ID`, `GCP_WIF_PROVIDER`, and `GCP_WIF_SA_EMAIL` (the latter is available from `terraform output github_actions_sa_email`). Update repository vars that are outside Terraform ownership in this repo, especially `AWS_ROLE_ARN`, `AWS_WIF_AUDIENCE`, bucket/index names, `TURSO_DB_URL` when Turso is enabled, CORS origins, contact email, and any Databricks settings.
5. Migrate Firestore data explicitly. The app switches to the new database as soon as `GCP_PROJECT_ID` changes, so export from the source project and import into `dastill` before frontend/backend cutover. Example shape:

```bash
gcloud firestore export gs://<shared-migration-bucket>/<export-prefix> \
  --project=<source-project-id>

gcloud firestore import gs://<shared-migration-bucket>/<export-prefix> \
  --project=dastill
```

6. Enable Firebase on the new project, set any optional Firebase Terraform inputs you need such as `firebase_authorized_domains_extra`, then re-apply Terraform so it creates the web app, refreshes Secret Manager with the effective frontend Firebase values, and updates authorized domains through Identity Platform. Keep Google sign-in configuration in `frontend/firebase.json` and deploy it separately with `bunx firebase-tools@15.12.0 deploy --only auth --project "$PROJECT_ID" --config frontend/firebase.json --non-interactive`.
7. Re-run the release workflow after the GitHub secret/var cutover so Cloud Run revisions pick up the new project ID, Firebase config, backend URL, and docs URL.

## CI/CD Flow

The GitHub Actions workflow:

```text
1. Builds and pushes backend, docs, and frontend images to Artifact Registry
2. Deploys backend, docs, and frontend to Cloud Run (main branch or release dispatch)
3. Resolves deployed backend and docs URLs for the frontend service env
4. Deploys the backend with runtime env including S3/AWS config, `TURSO_DB_URL`, and Secret Manager mounts such as `TURSO_AUTH_TOKEN` when enabled
5. Deploys the frontend with runtime env including BACKEND_API_BASE, BACKEND_IDENTITY_AUDIENCE, PUBLIC_DOCS_URL, PUBLIC_CONTACT_EMAIL, PUBLIC_FIREBASE_PROJECT_ID, and Firebase client values from Secret Manager mounts
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

Keyword search can also use Turso/libSQL for durable FTS storage. In that setup:

- set `turso_auth_token` in `terraform.tfvars` and run `terraform apply`
- set GitHub repository variable `TURSO_DB_URL=libsql://...`
- rerun the Release workflow so Cloud Run mounts `TURSO_AUTH_TOKEN` and passes `TURSO_DB_URL`

### Search vector index

ANN index creation is intentionally not part of startup migrations because it is too expensive for remote bulk indexing workflows.

### Docs frontend

The docs site is deployed as its own Cloud Run service from `docs/Dockerfile`. It serves the static VitePress build through nginx and remains operationally separate from the product frontend.

The `main`-branch deploy workflow publishes the docs revision with unauthenticated access enabled, so the service is reachable immediately after each successful deployment.

The product frontend links to this docs service through a `PUBLIC_DOCS_URL` runtime env var on the frontend Cloud Run service. Local development falls back to `http://localhost:4173` when that variable is unset.

Terraform grants the GitHub Actions deploy identity Cloud Run admin permissions and service-account-user bindings so the workflow can keep managing all three Cloud Run services.

## Billing Export

Terraform can optionally create the BigQuery prerequisites for Cloud Billing export:

- dataset `billing_export` by default
- configurable export project and dataset location
- dataset access for Google-managed billing export writers
- required BigQuery APIs

Enable this by setting `billing_export_enabled = true` in `terraform.tfvars` and optionally overriding `billing_export_project_id`, `billing_export_dataset_id`, or `billing_export_dataset_location`.

After `terraform apply`, finish the setup in Cloud Billing by opening the billing account linked to the project, navigating to **Billing export**, and pointing the detailed usage export at the Terraform-managed dataset. Terraform does not manage that final toggle because Cloud Billing does not expose it as a supported first-class Terraform resource.
