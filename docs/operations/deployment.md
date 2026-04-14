# Deployment and Operations

## Current Production Shape

The repository now runs one runtime service and two static Hosting targets:

- backend on Cloud Run
- product frontend on Firebase Hosting
- docs frontend on Firebase Hosting

## Infrastructure Ownership

Terraform manages:

- Cloud Run backend (GCP)
- Firebase project resources, the web app, and the docs Hosting site (GCP)
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

- `OPEN_ALEX_API_KEY`
- `OLLAMA_API_KEY`
- `YOUTUBE_API_KEY`
- `LOGFIRE_TOKEN` (when Logfire observability is enabled for the backend)
- `BACKEND_PROXY_TOKEN`
- `TURSO_AUTH_TOKEN` (when Turso-backed keyword search is enabled in production)
- `DATABRICKS_TOKEN` (only when Databricks ingestion is configured)
- `firebase_web_api_key` and `firebase_auth_domain` (product frontend; Terraform derives both from the Firebase web app config and writes them to Secret Manager)

`YOUTUBE_API_KEY` is project-scoped. When the target GCP project changes, create a fresh key in that project, update `youtube_api_key` in `terraform.tfvars`, run `terraform apply`, and redeploy the backend so Cloud Run mounts the new Secret Manager version.

`OPEN_ALEX_API_KEY` should be managed the same way for production. Put it in `terraform.tfvars` as `openalex_api_key`, run `terraform apply`, and redeploy the backend so Cloud Run mounts the latest Secret Manager version.

Non-secret backend runtime config is passed as plain env values for:

- `AWS_REGION`
- `S3_DATA_BUCKET`
- `S3_VECTOR_BUCKET`
- `S3_VECTOR_INDEX`
- `START_APP_USE_TURSO` (production should set this to enable the shared Turso replica instead of per-instance local libSQL)
- `TURSO_DB_URL` (when Turso-backed keyword search is enabled in production)
- `AWS_ROLE_ARN` (production only)
- `AWS_WIF_AUDIENCE` (production only)
- `OLLAMA_URL`
- `OLLAMA_SUMMARY_MODEL`
- `OLLAMA_FALLBACK_MODEL`
- `OLLAMA_DEFAULT_CHAT_MODEL`
- `OLLAMA_EMBEDDING_MODEL`
- `SUMMARY_EVALUATOR_MODEL`
- `DATABRICKS_HOST` (when Databricks ingestion is enabled)
- `DATABRICKS_WAREHOUSE_ID` (when Databricks ingestion is enabled)
- `DATABRICKS_CATALOG` (when Databricks ingestion is enabled)
- `DATABRICKS_SCHEMA` (when Databricks ingestion is enabled)
- `SUMMARIZE_PATH`
- log level

Non-secret product frontend runtime config is passed as plain env values for:

- build-time `VITE_API_BASE`
- build-time `PUBLIC_DOCS_URL`
- build-time `PUBLIC_FIREBASE_PROJECT_ID`
- optional build-time `PUBLIC_BROWSER_AUTH_BASE_URL`
- build-time `PUBLIC_CONTACT_EMAIL`

### Firebase Auth (product frontend)

The frontend uses the Firebase JS SDK in the browser and in the Tauri WebView. Signed-in requests send the Firebase ID token directly to the backend as `Authorization: Bearer <token>`. The web client reads **`PUBLIC_FIREBASE_API_KEY`**, **`PUBLIC_FIREBASE_AUTH_DOMAIN`**, and **`PUBLIC_FIREBASE_PROJECT_ID`** at build time.

**Terraform (`terraform.tfvars`, not GitHub Variables):** Terraform creates the Firebase project resources and web app, then reads the effective Web API key and auth domain from the Firebase web app config data source before writing them to Secret Manager. Run `terraform apply` so secrets `dastill-firebase-web-api-key` and `dastill-firebase-auth-domain` exist and the GitHub Actions deploy identity can read them during Hosting builds.

**Google sign-in:** anonymous auth stays enabled through Identity Platform. Google sign-in itself is managed through the repo-root [`firebase.json`](../../firebase.json) and should be deployed separately with `bunx firebase-tools@15.12.0 deploy --only auth --project "$PROJECT_ID" --non-interactive` when provisioning a new project or when that file changes. That lets Firebase provision the correct project-local Google OAuth client for the web app instead of reusing a copied client ID/secret from another project.

**Release workflow:** resolves the Terraform-managed frontend Firebase secrets `dastill-firebase-web-api-key` and `dastill-firebase-auth-domain` before building the static frontend bundle. It passes those values into the build together with `VITE_API_BASE`, `PUBLIC_DOCS_URL`, `PUBLIC_CONTACT_EMAIL`, and `PUBLIC_FIREBASE_PROJECT_ID`. Routine releases do not redeploy Firebase Auth config.

**Android browser-auth handoff:** if the Tauri Android shell should open a browser-hosted login page on a different origin than the product frontend itself, set `PUBLIC_BROWSER_AUTH_BASE_URL` for the frontend build. That value controls the origin used for the system-browser `/login` handoff flow.

**Authorized domains:** Terraform manages Identity Platform authorized domains. The default set includes `localhost`, the Firebase-hosted domains for the project, and any entries in `firebase_authorized_domains_extra`. Use Terraform rather than console-only edits for managed environments.

## Project Migration

To cut over from a previous GCP project to the current `dastill` project:

1. Create or gain access to the `dastill` GCP project, attach billing, and decide the Firestore location before the first apply. The repo now exposes `firestore_location_id` explicitly; the example uses `eur3`.
2. Update your local `terraform.tfvars` for the new target project. Set `project_id = "dastill"` and keep `app_name = "dastill"` unless you intentionally want new GCP/AWS resource names. If the GitHub Workload Identity Pool lives outside the target project, also set `github_wif_pool_project_number`; otherwise it defaults to the active project number.
3. Decide how Terraform state will handle the shared AWS resources. Buckets, vector buckets, and the `dastill-gcp-backend` AWS role are keyed by `app_name`, not `project_id`. Reusing the existing state is the simplest cutover. If you start from a fresh state backend, import the existing AWS resources before apply or intentionally rename `app_name` and migrate that data separately.
4. Apply Terraform against the new project and record the outputs you need for GitHub. At minimum, update repository secrets `GCP_PROJECT_ID`, `GCP_WIF_PROVIDER`, and `GCP_WIF_SA_EMAIL` (the latter is available from `terraform output github_actions_sa_email`). Update repository vars that are outside Terraform ownership in this repo, especially `AWS_ROLE_ARN`, `AWS_WIF_AUDIENCE`, bucket/index names, `TURSO_DB_URL` when Turso is enabled, CORS origins, contact email, and any Databricks settings.
5. Rotate project-local API keys and tokens before cutover. In particular, create a fresh `YOUTUBE_API_KEY` in the new GCP project, update both local `~/.config/dastill/backend.env` and Terraform-managed `youtube_api_key` in `terraform.tfvars`, then `terraform apply` so Secret Manager and Cloud Run stop using the previous project's key.
6. Migrate Firestore data explicitly. The app switches to the new database as soon as `GCP_PROJECT_ID` changes, so export from the source project and import into `dastill` before frontend/backend cutover. Example shape:

```bash
gcloud firestore export gs://<shared-migration-bucket>/<export-prefix> \
  --project=<source-project-id>

gcloud firestore import gs://<shared-migration-bucket>/<export-prefix> \
  --project=dastill
```

7. Enable Firebase on the new project, set any optional Firebase Terraform inputs you need such as `firebase_authorized_domains_extra`, then re-apply Terraform so it creates the web app, the docs Hosting site, refreshes Secret Manager with the effective frontend Firebase values, and updates authorized domains through Identity Platform. Keep Google sign-in configuration in the repo-root `firebase.json` and deploy it separately with `bunx firebase-tools@15.12.0 deploy --only auth --project "$PROJECT_ID" --non-interactive`.
8. Re-run the release workflow after the GitHub secret/var cutover so the backend Cloud Run service and the frontend/docs Hosting targets pick up the new project ID, Firebase config, backend URL, docs URL, and the latest Secret Manager versions.

## CI/CD Flow

The GitHub Actions workflow:

```text
1. Runs repo hygiene on every validation run
2. Detects which of `backend/`, `frontend/`, `docs/`, and the root Firebase Hosting config changed
3. Runs only the matching backend/frontend/docs validation jobs on push and pull request events
4. On `main`, builds and deploys only the surfaces with deploy-relevant changes
5. Skips deploys for trivial-only service changes such as `.gitignore`, README, and test-only frontend/backend changes
6. Resolves the backend Cloud Run URL and a stable docs Hosting URL when the frontend itself is being deployed
7. Deploys the backend with runtime env including S3/AWS config, `START_APP_USE_TURSO=1`, `TURSO_DB_URL`, and Secret Manager mounts such as `TURSO_AUTH_TOKEN` when enabled
8. Builds the static frontend and docs bundles with Bun, then deploys them with Firebase CLI using Workload Identity Federation credentials
9. Builds Android APK artifacts through `.github/workflows/android.yml` when mobile changes are pushed or manually requested
```

## Docker Layout

### Backend image

- built from `backend/Dockerfile`
- compiles Rust in a builder stage
- runs the `dastill` binary in a slim Debian runtime image
- bundles a `summarize` script path for transcript extraction

### Frontend and docs bundles

- the frontend is built in CI from `frontend/` with Bun and published from `frontend/build`
- the docs site is built in CI from `docs/` with Bun and published from `docs/.vitepress/dist`
- Firebase Hosting serves both static outputs directly, so there are no frontend/docs runtime containers in production

### Android artifacts

- built from the Tauri v2 project in `src-tauri/`
- use `.github/workflows/android.yml`
- resolve backend/docs/Firebase build values before running `cargo tauri android build`
- upload a release APK as a workflow artifact

## Operational Notes

### Search in production

Production defaults to plain FTS mode unless `SEARCH_SEMANTIC_ENABLED=true` is intentionally set.

Keyword search can also use Turso/libSQL for durable FTS storage via direct remote queries. In that setup:

- production Cloud Run should set `START_APP_USE_TURSO=1`
- set `turso_auth_token` in `terraform.tfvars` and run `terraform apply`
- set GitHub repository variable `TURSO_DB_URL=libsql://...`
- rerun the Release workflow so Cloud Run mounts `TURSO_AUTH_TOKEN` and passes `TURSO_DB_URL`

### Search vector index

ANN index creation is intentionally not part of startup migrations because it is too expensive for remote bulk indexing workflows.

### Docs frontend

The docs site is deployed as its own Firebase Hosting site and remains operationally separate from the product frontend.

The `main`-branch deploy workflow publishes the docs Hosting revision directly from the repo root, so the site is reachable immediately after each successful deployment.

The product frontend links to this docs site through a build-time `PUBLIC_DOCS_URL`. Local development still falls back to `http://localhost:4173` when that variable is unset.

Terraform grants the GitHub Actions deploy identity the Cloud Run permissions needed for the backend plus Firebase Hosting permissions for the static sites.

## Billing Export

Terraform can optionally create the BigQuery prerequisites for Cloud Billing export:

- dataset `billing_export` by default
- configurable export project and dataset location
- dataset access for Google-managed billing export writers
- required BigQuery APIs

Enable this by setting `billing_export_enabled = true` in `terraform.tfvars` and optionally overriding `billing_export_project_id`, `billing_export_dataset_id`, or `billing_export_dataset_location`.

After `terraform apply`, finish the setup in Cloud Billing by opening the billing account linked to the project, navigating to **Billing export**, and pointing the detailed usage export at the Terraform-managed dataset. Terraform does not manage that final toggle because Cloud Billing does not expose it as a supported first-class Terraform resource.
