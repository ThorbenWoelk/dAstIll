# Deployment and Operations

## Production Surfaces

dAstIll deploys these surfaces:

| Surface     | Runtime          | Deployed from                   |
| ----------- | ---------------- | ------------------------------- |
| Backend API | Cloud Run        | `backend/Dockerfile`            |
| Podcast ASR | Cloud Run        | `asr/Dockerfile`                |
| Frontend UI | Firebase Hosting | `frontend/build`                |
| Docs UI     | Firebase Hosting | `docs/.vitepress/dist`          |
| Android APK | GitHub artifact  | `.github/workflows/android.yml` |

## Infrastructure as Code (IaC)

Terraform manages:

- Cloud Run backend service
- optional Cloud Run podcast ASR service inputs
- Firebase project resources, web app, frontend Hosting, and docs Hosting
- GCP service accounts and IAM
- GCP Secret Manager secret containers and IAM bindings
- AWS S3 bucket for data storage
- AWS S3 Vectors bucket and index for semantic search
- AWS IAM role for GCP Workload Identity Federation
- GitHub Actions infrastructure permissions
- optional BigQuery billing export prerequisites
- optional Cloud Billing alert budgets

Terraform owns containers, identities, permissions, and non-secret runtime config. It does not own
secret payloads.

The billing export and billing budget resources are optional because dAstIll can deploy and run
without them. They are enabled only when the matching Terraform variables are set, so local,
staging, or personal projects can skip extra billing-account permissions and cost-reporting setup.

## Cross-Cloud Authentication

### Backend Runtime

The backend runs on Cloud Run and accesses AWS S3 and S3 Vectors through GCP to AWS Workload
Identity Federation.

1. The AWS role `dastill-gcp-backend` trusts the backend GCP service account.
2. Cloud Run receives the backend AWS federation config.
3. The backend exchanges its GCP identity for AWS temporary credentials.
4. S3 and S3 Vectors calls use those AWS credentials.

Local development uses shared machine-local AWS credentials. See
[Local Development](/operations/local-development).

### Infra CI

Terraform in GitHub Actions uses a separate trust path.

1. GitHub Actions requests an OIDC token from `token.actions.githubusercontent.com`.
2. AWS trusts that provider for this repository.
3. `infra.yml` assumes the AWS role `dastill-github-terraform`.
4. Terraform applies AWS resources with short-lived CI credentials.

GCP CI auth uses GitHub OIDC to GCP Workload Identity Federation and the `dastill-github-sa`
service account.

Do not reuse the backend runtime AWS role for GitHub CI.

## Secrets

Production secrets live in GCP Secret Manager. Terraform creates the secret containers and IAM
bindings. Add payloads directly in Secret Manager.

Backend/runtime secret containers:

- `dastill-youtube-api-key`
- `dastill-openalex-api-key`
- `dastill-ollama-api-key`
- `dastill-logfire-token`
- `dastill-backend-proxy-token`
- `dastill-databricks-token`

Frontend build secret containers:

- `dastill-firebase-web-api-key`
- `dastill-firebase-auth-domain`

`infra.yml` refreshes the Firebase frontend build secrets after Terraform apply by reading the
effective Firebase web app config. Other app secrets are manual Secret Manager version adds.

### Add Or Rotate A Secret

Use this flow for a new project, a new secret container, or a rotation.

1. Apply Terraform so the container and IAM binding exist.
2. Add the secret payload as a new Secret Manager version.
3. Redeploy the surfaces that consume the secret.

Example:

```bash
gcloud secrets versions add <secret-name> \
  --project <project-id> \
  --data-file=<payload-file>
```

Some API keys are project-scoped. When the target GCP project changes, create fresh keys in that
project, add new Secret Manager versions, and redeploy the backend.

Bootstrap edge: the first creation of the AWS GitHub OIDC provider and `dastill-github-terraform`
role must happen from an already authenticated AWS context. After that, CI owns the recurring
Terraform path.

### Retire A Secret

Secret lifecycle stays in IaC.

1. Remove app/runtime usage.
2. Remove deploy-time references in `.github/workflows/deploy.yml` and
   `.github/workflows/infra.yml`.
3. Remove IAM references in `terraform/iam.tf`.
4. Remove the secret resource in `terraform/secrets.tf`.
5. Update runbooks that still mention the secret.
6. Apply Terraform.

## Runtime Config

Sensitive values come from Secret Manager. Non-secret production config is passed as Cloud Run or
build-time env values.

Backend runtime config includes:

- AWS storage values
- AWS federation values
- search, model, reranker, HyDE, and chat guardrail values
- ingestion values
- optional ASR values
- optional Databricks values
- log level

Frontend build config includes:

- backend API base URL
- docs URL
- Firebase public config
- contact/support config
- maintenance-mode config
- optional browser-auth origin for mobile handoff

See `backend/.env.example` and `frontend/.env.example` for the current key names.
Runtime limits, quotas, cooldowns, and timeout values live in
[Runtime Limits](/operations/runtime-limits).

## Firebase Auth

The frontend uses Firebase Auth in the browser and in the Tauri WebView. Signed-in requests
send the Firebase ID token directly to the backend as `Authorization: Bearer <token>`.

Terraform creates Firebase project resources, the web app, secret containers, and IAM. The infra
workflow reads the effective Web API key and auth domain from the Firebase Management API and syncs
them to Secret Manager without storing those values in Terraform state.

Google sign-in is managed through [`firebase.json`](../../firebase.json). Deploy it when provisioning
a new project or when the file changes:

```bash
bunx firebase-tools@15.12.0 deploy \
  --only auth \
  --project <project-id> \
  --non-interactive
```

Terraform manages Identity Platform authorized domains. Use `firebase_authorized_domains_extra` for
managed additions.

## Runtime Mode

`.github/runtime-mode.env` controls release posture.

When release maintenance mode is enabled, CI builds the frontend in maintenance mode. The
backend still validates and deploys so `dastill-mini` remains available.

## Backend Runtime Boundary

The backend keeps a local libSQL cache/index and in-process workers. The production instance cap and
the scale-out boundary live in [Runtime Limits](/operations/runtime-limits#deployment-capacity).

On startup, the backend restores or rebuilds its local libSQL file from S3-backed runtime cache
objects.

## Podcast ASR

Podcast ASR is a separate OpenAI-compatible transcription service:

```text
POST /v1/audio/transcriptions
```

When production ASR is enabled, the release workflow builds `asr/Dockerfile` and deploys the ASR
Cloud Run service. The repo-owned production path uses Cloud Run IAM and does not need a shared
service API key.

For the repo-owned Cloud Run path, the backend sends the validated public audio URL to ASR. The ASR
service fetches the audio, converts it with `ffmpeg`, and transcribes it with `whisper.cpp`.

The ASR service capacity and request timeout live in
[Runtime Limits](/operations/runtime-limits#deployment-capacity).

Content-pipeline behavior is owned by [Content Pipeline](/pipelines/content-pipeline).

## Static Sites And Android

The frontend and docs are static Firebase Hosting targets. There are no frontend or docs runtime
containers in production.

The frontend links to the docs site through build-time frontend config.

Android release APKs are built by `.github/workflows/android.yml`. The workflow resolves deployed
backend/docs URLs and Firebase frontend build values before running `cargo tauri android build`.

## CI/CD Flow

The GitHub Actions workflows:

1. Run repo hygiene on validation runs.
2. Run `infra.yml` for `terraform/**` changes: fmt, validate, plan on PRs, apply on `main`.
3. Sync Firebase frontend config into Secret Manager after Terraform apply.
4. Detect changed surfaces across `backend/`, `frontend/`, `docs/`, and root Firebase Hosting config.
5. Run only matching backend/frontend/docs validation jobs.
6. On `main`, wait for infra apply before app deployment when the same push touched `terraform/**`.
7. Build and deploy only surfaces with deploy-relevant changes.
8. Resolve backend Cloud Run URL and stable docs Hosting URL when the frontend is deployed.
9. Deploy the backend with runtime env and app/runtime secrets.
10. Build the static frontend and docs bundles with Bun and deploy them with Firebase CLI.
11. Build Android APK artifacts when mobile changes are pushed or the workflow is manually requested.

## Billing Export

Terraform can create BigQuery prerequisites for Cloud Billing export:

- dataset `billing_export` by default
- configurable export project and dataset location
- dataset access for Google-managed billing export writers
- required BigQuery APIs

Enable this with `billing_export_enabled = true` in `terraform.tfvars`. You can also override
`billing_export_project_id`, `billing_export_dataset_id`, and `billing_export_dataset_location`.

When `billing_export_enabled` is false, Terraform does not create the billing export dataset or
enable the BigQuery billing export APIs. Runtime services are unaffected because billing export is
only for cost analysis.

After `terraform apply`, finish setup in Cloud Billing. Open the billing account, go to
**Billing export**, and point detailed usage export at the Terraform-managed dataset. Terraform does
not manage that final toggle.

## Billing Budgets

Terraform can create monthly alert budgets when `billing_budgets_enabled = true`:

- one all-service budget for each configured dAstIll project
- one Cloud Run service-scoped budget for each configured dAstIll project

The primary `project_id` is always included. Add other projects that run dAstIll Cloud Run services
to `billing_budget_project_ids`.

When `billing_budgets_enabled` is false, Terraform does not create budget resources or enable the
Cloud Billing Budget API. Runtime services are unaffected because budgets only send alerts; they do
not cap, stop, or throttle spend.

Budget amounts and thresholds live in
[Runtime Limits](/operations/runtime-limits#billing-alert-budgets).

Budget creation needs the Cloud Billing Budget API and budget write permissions. For CI, make sure
the Terraform identity can manage budgets for the target billing account or single-project budgets
for each configured project.
