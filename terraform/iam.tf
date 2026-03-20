resource "google_service_account" "backend_sa" {
  account_id   = "${var.app_name}-backend-sa"
  display_name = "${var.app_name} Backend Service Account"
}

resource "google_service_account" "frontend_sa" {
  account_id   = "${var.app_name}-frontend-sa"
  display_name = "${var.app_name} Frontend Service Account"
}

resource "google_service_account" "docs_sa" {
  account_id   = "${var.app_name}-docs-sa"
  display_name = "${var.app_name} Docs Service Account"
}

# Service Account for GitHub Actions
resource "google_service_account" "github_actions_sa" {
  account_id   = "${var.app_name}-github-sa"
  display_name = "${var.app_name} GitHub Actions Service Account"
}

# Grant access to secrets for runtime services and GitHub Actions (deploy-time binding)
locals {
  backend_secret_ids = {
    ollama_api_key      = google_secret_manager_secret.ollama_api_key.id
    youtube_api_key     = google_secret_manager_secret.youtube_api_key.id
    logfire_token       = google_secret_manager_secret.logfire_token.id
    backend_proxy_token = google_secret_manager_secret.backend_proxy_token.id
  }
  frontend_secret_ids = {
    app_auth_password   = google_secret_manager_secret.app_auth_password.id
    app_session_secret  = google_secret_manager_secret.app_session_secret.id
    backend_proxy_token = google_secret_manager_secret.backend_proxy_token.id
  }
  cicd_secret_ids = merge(local.backend_secret_ids, local.frontend_secret_ids)
}

resource "google_secret_manager_secret_iam_member" "backend_secrets" {
  for_each  = local.backend_secret_ids
  secret_id = each.value
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.backend_sa.email}"
}

resource "google_secret_manager_secret_iam_member" "frontend_secrets" {
  for_each  = local.frontend_secret_ids
  secret_id = each.value
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.frontend_sa.email}"
}

resource "google_secret_manager_secret_iam_member" "cicd_secrets" {
  for_each  = local.cicd_secret_ids
  secret_id = each.value
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_cloud_run_v2_service_iam_member" "backend_frontend_invoker" {
  location = google_cloud_run_v2_service.backend.location
  name     = google_cloud_run_v2_service.backend.name
  role     = "roles/run.invoker"
  member   = "serviceAccount:${google_service_account.frontend_sa.email}"
}

# CICD Permissions
resource "google_artifact_registry_repository_iam_member" "repo_writer" {
  location   = google_artifact_registry_repository.repo.location
  repository = google_artifact_registry_repository.repo.name
  role       = "roles/artifactregistry.writer"
  member     = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_project_iam_member" "cloud_run_admin" {
  project = var.project_id
  role    = "roles/run.admin"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

output "backend_sa_unique_id" {
  value = google_service_account.backend_sa.unique_id
}

resource "google_service_account_iam_member" "sa_user_backend" {
  service_account_id = google_service_account.backend_sa.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_service_account_iam_member" "sa_user_frontend" {
  service_account_id = google_service_account.frontend_sa.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_service_account_iam_member" "sa_user_docs" {
  service_account_id = google_service_account.docs_sa.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

# Needed for gcloud auth configure-docker if using impersonation/WIF in some contexts
resource "google_project_iam_member" "token_creator" {
  project = var.project_id
  role    = "roles/iam.serviceAccountTokenCreator"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

# For WIF
resource "google_service_account_iam_member" "wif_user" {
  service_account_id = google_service_account.github_actions_sa.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/projects/673062863574/locations/global/workloadIdentityPools/github-pool-v1/attribute.repository/ThorbenWoelk/dAstIll"
}
