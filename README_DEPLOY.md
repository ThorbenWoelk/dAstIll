# Deployment Instructions for dAstIll

This project is set up to be deployed on **Google Cloud Run** using **Terraform** for IaC and **GitHub Actions** for CI/CD.

## Versioning

The project uses unified versioning (`0.1.0`) across:
- `backend/Cargo.toml`
- `frontend/package.json`
- UI displays and documentation.

## Prerequisites

1.  A GCP Project.
2.  `gcloud` CLI installed and authenticated.
3.  `terraform` CLI installed.
4.  A Turso database (for `libsql`).

## Step 1: Infrastructure Setup

1.  Navigate to the `terraform/` directory.
2.  Create a `terraform.tfvars` file (use `terraform.tfvars.example` as a template).
3.  Initialize Terraform:
    ```bash
    terraform init
    ```
4.  Apply the configuration:
    ```bash
    terraform apply
    ```
    This will create:
    - Artifact Registry repository.
    - Secret Manager secrets (for Turso and YouTube).
    - Service Accounts with appropriate permissions.
    - Cloud Run services (initially running a hello-world image).

## Step 2: GitHub Actions Configuration

1.  In your GitHub repository, go to **Settings > Secrets and variables > Actions**.
2.  Add the following secrets:
    - `GCP_PROJECT_ID`: Your GCP project ID.
    - `GCP_WIF_PROVIDER`: The full name of your Workload Identity Provider (e.g., `projects/123456789/locations/global/workloadIdentityPools/my-pool/providers/my-provider`).
    - `GCP_WIF_SA_EMAIL`: The email of the service account used for GitHub Actions (needs permissions to deploy to Cloud Run and push to Artifact Registry).

## Step 3: Deployment

Once secrets are configured, any push to the `main` branch will trigger the `Deploy` workflow:
1.  Builds and pushes the Rust backend image.
2.  Deploys the backend to Cloud Run.
3.  Builds the SvelteKit frontend image (passing the backend URL as a build argument).
4.  Deploys the frontend to Cloud Run.

## Note on the `summarize` tool

The backend expects a `summarize` binary for transcript extraction (defined by `SUMMARIZE_PATH`).
The current `backend/Dockerfile` installs `yt-dlp` as a dependency. If your `summarize` tool is a custom binary, you should:
1.  Update the `backend/Dockerfile` to include its source or build it.
2.  Or update `SUMMARIZE_PATH` in Terraform to point to a script that wraps `yt-dlp` if it's compatible.
