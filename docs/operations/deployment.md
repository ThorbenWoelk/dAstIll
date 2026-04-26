# Deployment and Operations

## Production Shape

The repository runs one primary runtime service, one optional ASR runtime service, and two static Hosting targets:

- backend on Cloud Run
- podcast ASR on Cloud Run when `LOCAL_ASR_ENABLED=true`
- product frontend on Firebase Hosting
- docs frontend on Firebase Hosting

## Infrastructure Ownership

Terraform manages:

- Cloud Run backend (GCP)
- Firebase project resources, the web app, and the docs Hosting site (GCP)
- service accounts and IAM (GCP and AWS)
- optional Cloud Billing alert budgets for dAstIll project spend and Cloud Run spend
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

Local development uses the shared machine-local AWS credentials files documented in [Local Development](/operations/local-development), with inline environment credentials as a fallback.

Terraform in GitHub Actions uses a separate AWS trust path:

1. GitHub Actions requests an OIDC token from `token.actions.githubusercontent.com`
2. AWS IAM trusts that OIDC provider for this repository
3. `infra.yml` assumes the Terraform role `dastill-github-terraform`
4. Terraform AWS resources then run with short-lived AWS credentials in CI

This is separate from the Cloud Run backend runtime role. Do not reuse the backend runtime role for GitHub CI.

## Secret and Config Boundaries

Secrets are stored in GCP Secret Manager for:

- `OPEN_ALEX_API_KEY`
- `OLLAMA_API_KEY`
- `YOUTUBE_API_KEY`
- `LOGFIRE_TOKEN` (when Logfire observability is enabled for the backend)
- `BACKEND_PROXY_TOKEN`
- `DATABRICKS_TOKEN` (only when Databricks ingestion is configured)
- `firebase_web_api_key` and `firebase_auth_domain` (product frontend; the infra workflow derives both from the Firebase web app config and syncs them to Secret Manager after apply)

Terraform owns the secret containers and IAM bindings only. Add or rotate secret payloads directly in Secret Manager with `gcloud secrets versions add ...` or the Cloud Console; do not put app credentials in `terraform.tfvars`.

`YOUTUBE_API_KEY` is project-scoped. When the target GCP project changes, create a fresh key in that project, add a new Secret Manager version for `dastill-youtube-api-key`, and redeploy the backend so Cloud Run mounts the new version.

`OPEN_ALEX_API_KEY` should be managed the same way for production. Add a new Secret Manager version for `dastill-openalex-api-key` and redeploy the backend so Cloud Run mounts the latest version.

### Secret bootstrap and rotation

Use this flow when provisioning a new project, filling a newly created secret container, or rotating an existing value.

1. Apply Terraform first so the secret containers and IAM bindings exist.
2. Add the secret payload as a new Secret Manager version.
3. Redeploy the surfaces that consume that secret.

Bootstrap edge: the first creation of the AWS GitHub OIDC provider and Terraform role must happen from an already authenticated AWS context, because GitHub Actions cannot assume a role that does not exist yet. After that first apply, CI owns the recurring Terraform path.

Example shape:

```bash
printf '%s' "$YOUTUBE_API_KEY" | \
  gcloud secrets versions add dastill-youtube-api-key \
    --project "$PROJECT_ID" \
    --data-file=-
```

Backend/runtime secrets expected by infra and release workflows:

- `dastill-youtube-api-key`
- `dastill-openalex-api-key`
- `dastill-ollama-api-key`
- `dastill-logfire-token`
- `dastill-backend-proxy-token`
- `dastill-databricks-token`

Frontend build secrets:

- `dastill-firebase-web-api-key`
- `dastill-firebase-auth-domain`

The Firebase frontend secrets are refreshed automatically by `infra.yml` after Terraform apply. Do not hand-maintain them unless infra automation is unavailable.

### Secret deprecation

Secret lifecycle stays in IaC. Do not delete or rename production secrets only in the Cloud Console.

When retiring a secret:

1. Remove app/runtime usage first.
2. Remove deploy-time references from workflows such as `.github/workflows/deploy.yml` and `.github/workflows/infra.yml`.
3. Remove IAM references from `terraform/iam.tf`.
4. Remove the secret resource from `terraform/secrets.tf`.
5. Update this document and any runbooks that still mention the secret.
6. Apply Terraform so the managed secret container is destroyed or the IaC state matches the new desired posture.

If you need a safer staged retirement, first remove all consumers but keep the Terraform resource with a short `deprecated` comment for one release cycle, then delete the resource in a follow-up Terraform change. Keep the source of truth in IaC.

Non-secret backend runtime config is passed as plain env values for:

- `AWS_REGION`
- `S3_DATA_BUCKET`
- `S3_VECTOR_BUCKET`
- `S3_VECTOR_INDEX`
- `SEARCH_SEMANTIC_ENABLED`
- `SEARCH_AUTO_CREATE_VECTOR_INDEX`
- `DEFAULT_SEEDED_CHANNEL_IDS`
- `AWS_ROLE_ARN` (production only)
- `AWS_WIF_AUDIENCE` (production only)
- `OLLAMA_URL`
- `OLLAMA_SUMMARY_MODEL`
- `OLLAMA_FALLBACK_MODEL`
- `OLLAMA_DEFAULT_CHAT_MODEL`
- `OLLAMA_EMBEDDING_MODEL`
- `SEARCH_HYDE_MODEL`
- `SEARCH_RERANK_MODEL`
- `SUMMARY_EVALUATOR_MODEL`
- `CHAT_MULTI_PASS_ENABLED`
- `CHAT_GUARDRAIL_MODEL`
- `CHAT_PROMPT_BLOCKLIST`
- `CHAT_PROMPT_ALLOWLIST`
- `DATABRICKS_HOST` (when Databricks ingestion is enabled)
- `DATABRICKS_WAREHOUSE_ID` (when Databricks ingestion is enabled)
- `DATABRICKS_CATALOG` (when Databricks ingestion is enabled)
- `DATABRICKS_SCHEMA` (when Databricks ingestion is enabled)
- `DATABRICKS_BRONZE_TABLE` (when Databricks ingestion is enabled)
- `SUMMARIZE_PATH`
- `LOCAL_ASR_ENABLED`
- `LOCAL_ASR_BASE_URL`
- `LOCAL_ASR_AUTH_MODE`
- `LOCAL_ASR_MODEL`
- `LOCAL_ASR_MAX_AUDIO_BYTES`
- `LOCAL_ASR_TIMEOUT_SECS`
- log level

Production uses Cloud Run IAM between the backend and the repo-owned ASR service, so `LOCAL_ASR_API_KEY` is not required in the default production path. Use `LOCAL_ASR_API_KEY` only for local or externally hosted ASR endpoints that rely on bearer-token authentication.

Non-secret product frontend runtime config is passed as plain env values for:

- build-time `VITE_API_BASE`
- build-time `PUBLIC_DOCS_URL`
- build-time `PUBLIC_FIREBASE_PROJECT_ID`
- optional build-time `PUBLIC_BROWSER_AUTH_BASE_URL`
- build-time `PUBLIC_CONTACT_EMAIL`
- build-time `PUBLIC_APP_MAINTENANCE_MODE`

### Firebase Auth (product frontend)

The frontend uses the Firebase JS SDK in the browser and in the Tauri WebView. Signed-in requests send the Firebase ID token directly to the backend as `Authorization: Bearer <token>`. The web client reads **`PUBLIC_FIREBASE_API_KEY`**, **`PUBLIC_FIREBASE_AUTH_DOMAIN`**, and **`PUBLIC_FIREBASE_PROJECT_ID`** at build time.

**Terraform + infra CI:** Terraform creates the Firebase project resources, secret containers, and IAM. The infra workflow then reads the effective Web API key and auth domain from the Firebase Management API and syncs them to Secret Manager without storing them in Terraform state.

**Google sign-in:** anonymous auth stays enabled through Identity Platform. Google sign-in itself is managed through the repo-root [`firebase.json`](../../firebase.json) and should be deployed separately with `bunx firebase-tools@15.12.0 deploy --only auth --project "$PROJECT_ID" --non-interactive` when provisioning a new project or when that file changes. That lets Firebase provision the correct project-local Google OAuth client for the web app instead of reusing a copied client ID/secret from another project.

**Runtime mode source of truth:** the checked-in file `.github/runtime-mode.env` controls whether the repo is in normal or maintenance posture. CI and release workflows read that file before deciding whether backend validation/deploy should run and whether the frontend should build in maintenance mode.

**Release workflow:** resolves the frontend Firebase secrets `dastill-firebase-web-api-key` and `dastill-firebase-auth-domain` before building the static frontend bundle. It passes those values into the build together with `VITE_API_BASE`, `PUBLIC_DOCS_URL`, `PUBLIC_CONTACT_EMAIL`, `PUBLIC_APP_MAINTENANCE_MODE`, and `PUBLIC_FIREBASE_PROJECT_ID`. Routine releases do not redeploy Firebase Auth config.

When `APP_RUNTIME_MODE=maintenance` is set in `.github/runtime-mode.env`, the app frontend is built with `PUBLIC_APP_MAINTENANCE_MODE=1`, but the backend still validates and deploys. This keeps `dastill-mini` available while the main product UI stays in maintenance posture.

The backend Cloud Run service is intentionally capped at one serving instance. The runtime keeps a local libSQL cache/index plus in-process background workers, so multi-replica scale-out would duplicate worker execution and create per-replica cache divergence. Treat horizontal backend scaling as blocked until the serving path and worker path are split or otherwise coordinated.

On startup, the backend first tries to restore the local libSQL file from the derived S3 runtime cache at `runtime-cache/libsql/current.json`. The manifest points to a compressed snapshot under `runtime-cache/libsql/snapshots/` and includes source-prefix fingerprints for `videos/`, `user-preferences/`, and `tts-stats/`. If the snapshot is missing, stale, corrupt, or schema-incompatible, startup falls back to rebuilding local libSQL from the canonical S3 objects and then publishes a fresh derived snapshot. The canonical S3 prefixes remain the source of truth.

**Android browser-auth handoff:** if the Tauri Android shell should open a browser-hosted login page on a different origin than the product frontend itself, set `PUBLIC_BROWSER_AUTH_BASE_URL` for the frontend build. That value controls the origin used for the system-browser `/login` handoff flow.

**Authorized domains:** Terraform manages Identity Platform authorized domains. The default set includes `localhost`, the Firebase-hosted domains for the project, and any entries in `firebase_authorized_domains_extra`. Use Terraform rather than console-only edits for managed environments.

## Project Migration

To cut over from a previous GCP project to the current `dastill` project:

1. Create or gain access to the `dastill` GCP project and attach billing before the first apply.
2. Update your local `terraform.tfvars` for the new target project. Set `project_id = "dastill"` and keep `app_name = "dastill"` unless you intentionally want new GCP/AWS resource names. If the GitHub Workload Identity Pool lives outside the target project, also set `github_wif_pool_project_number`; otherwise it defaults to the active project number.
3. Decide how Terraform state will handle the shared AWS resources. Buckets, vector buckets, and the `dastill-gcp-backend` AWS role are keyed by `app_name`, not `project_id`. Reusing the existing state is the simplest cutover. If you start from a fresh state backend, import the existing AWS resources before apply or intentionally rename `app_name` and migrate that data separately.
4. Apply Terraform against the new project and record the outputs you need for GitHub. At minimum, update repository secrets `GCP_PROJECT_ID`, `GCP_WIF_PROVIDER`, and `GCP_WIF_SA_EMAIL` (the latter is available from `terraform output github_actions_sa_email`) and set repository variable `TERRAFORM_STATE_BUCKET` to the shared GCS state bucket name used by infra CI. Update repository vars that are outside Terraform ownership in this repo, especially `AWS_ROLE_ARN`, `AWS_WIF_AUDIENCE`, bucket/index names, CORS origins, contact email, and any Databricks settings.
5. Rotate project-local API keys and tokens before cutover. In particular, create a fresh `YOUTUBE_API_KEY` in the new GCP project, update both local `~/.config/dastill/backend.env` and the `dastill-youtube-api-key` secret in Secret Manager, then redeploy the backend so Cloud Run stops using the previous project's key.
6. The current data cutover boundary is storage-specific: local libSQL cache state is rebuilt from the S3-backed app data, and Firebase project changes mostly affect auth, Hosting, and project-local secrets/config.
7. Enable Firebase on the new project, set any optional Firebase Terraform inputs you need such as `firebase_authorized_domains_extra`, then re-apply Terraform so it creates the web app, the docs Hosting site, and updates authorized domains through Identity Platform. The infra workflow will refresh the frontend Firebase secrets in Secret Manager after apply. Keep Google sign-in configuration in the repo-root `firebase.json` and deploy it separately with `bunx firebase-tools@15.12.0 deploy --only auth --project "$PROJECT_ID" --non-interactive`.
8. Re-run the release workflow after the GitHub secret/var cutover so the backend Cloud Run service and the frontend/docs Hosting targets pick up the new project ID, Firebase config, backend URL, docs URL, and the latest Secret Manager versions.

## CI/CD Flow

The GitHub Actions workflows:

```text
1. Runs repo hygiene on every validation run
2. Runs `infra.yml` for `terraform/**` changes: fmt, validate, plan on PRs, apply on `main`; then syncs the Firebase frontend config into Secret Manager
   `infra.yml` authenticates to GCP through GitHub -> GCP Workload Identity Federation and to AWS through GitHub OIDC -> `dastill-github-terraform`
3. Detects which of `backend/`, `frontend/`, `docs/`, and the root Firebase Hosting config changed
4. Runs only the matching backend/frontend/docs validation jobs on push and pull request events
5. On `main`, waits for infra apply to finish before app deployment when the same push touched `terraform/**`
6. Builds and deploys only the surfaces with deploy-relevant changes
7. Skips deploys for trivial-only service changes such as `.gitignore`, README, and test-only frontend/backend changes
8. Resolves the backend Cloud Run URL and a stable docs Hosting URL when the frontend itself is being deployed
9. Deploys the backend with runtime env including S3/AWS config and the remaining app/runtime secrets
10. Builds the static frontend and docs bundles with Bun, then deploys them with Firebase CLI using Workload Identity Federation credentials
11. Builds Android APK artifacts through `.github/workflows/android.yml` when mobile changes are pushed or manually requested
```

## Docker Layout

### Backend image

- built from `backend/Dockerfile`
- compiles Rust in a builder stage
- runs the `dastill` binary in a slim Debian runtime image
- bundles a `summarize` script path for transcript extraction
- does not bundle the podcast STT model; podcast ASR runs in a separate operator-owned service

### Podcast ASR service

Podcast ASR is a separate service that implements the OpenAI-compatible
`POST /v1/audio/transcriptions` endpoint. The backend downloads validated public podcast audio and
posts it to this service. The repo-owned production service uses the maintained `whisper.cpp`
runtime with the `base.en` GGML model.

Keep the ASR service separate from the backend Cloud Run service so model files, CPU/GPU load, and
transcription failures do not affect the main API container. The repo-owned production service is invoked through Cloud Run IAM. Use `LOCAL_ASR_API_KEY` only for non-Cloud-Run or externally hosted ASR endpoints that need bearer-token auth.

The release workflow builds `asr/Dockerfile` and deploys a `${APP_NAME}-asr` Cloud Run service when
`LOCAL_ASR_ENABLED=true`. For this repo-owned Cloud Run path, the backend sends the validated public
audio URL to ASR instead of uploading the MP3 bytes through Cloud Run's request body limit. The ASR
service fetches the audio, converts it with `ffmpeg`, and transcribes it with `whisper.cpp`.

The service runs with 2 vCPU, 2 GiB memory, concurrency 1, max instances 1, and a 3600 second
timeout. Min instances stay at 0, so normal idle cost is zero; transcribing long episodes incurs
Cloud Run CPU, memory, request, egress, and image storage cost while the ASR instance is active. The
backend container does not load the model, but long podcast transcription requests can hold one
backend request open until durable ASR job state is added.

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

Keyword search uses the local libSQL FTS cache on each backend instance and rebuilds from `search-chunks/` when empty.

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

## Billing Budgets

Terraform can create monthly alert budgets when `billing_budgets_enabled = true`:

- one all-service budget for each configured dAstIll project
- one Cloud Run service-scoped budget for each configured dAstIll project

The primary `project_id` is always included. Add any other GCP projects that run dAstIll Cloud Run services to `billing_budget_project_ids`. Terraform looks up each project number, and the infra workflow resolves the primary project's billing account into `billing_budget_billing_account_id` before planning. Set `billing_budget_project_billing_account_ids` for any additional project that uses a different billing account.

Default alert levels are 50%, 80%, 100% actual spend, and 100% forecasted spend. The defaults are alert-only budgets of 50 billing-currency units for total project spend and 10 billing-currency units for Cloud Run spend. Adjust `billing_budget_app_monthly_amount_units`, `billing_budget_cloud_run_monthly_amount_units`, and `billing_budget_thresholds` for production thresholds.

Budget creation needs the Cloud Billing Budget API and budget write permissions. For CI, make sure the Terraform identity can manage budgets for the target billing account or single-project budgets for each configured project.
