resource "google_service_account" "backend_sa" {
  project      = var.project_id
  account_id   = "${var.app_name}-backend-sa"
  display_name = "${var.app_name} Backend Service Account"
}

# Service Account for GitHub Actions
resource "google_service_account" "github_actions_sa" {
  project      = var.project_id
  account_id   = "${var.app_name}-github-sa"
  display_name = "${var.app_name} GitHub Actions Service Account"
}

# Grant access to secrets for runtime services and GitHub Actions (deploy-time binding)
locals {
  backend_secret_ids = {
    openalex_api_key    = google_secret_manager_secret.openalex_api_key.id
    ollama_api_key      = google_secret_manager_secret.ollama_api_key.id
    youtube_api_key     = google_secret_manager_secret.youtube_api_key.id
    logfire_token       = google_secret_manager_secret.logfire_token.id
    backend_proxy_token = google_secret_manager_secret.backend_proxy_token.id
    databricks_token    = google_secret_manager_secret.databricks_token.id
    local_asr_api_key   = google_secret_manager_secret.local_asr_api_key.id
  }
  frontend_build_secret_ids = {
    firebase_web_api_key = google_secret_manager_secret.firebase_web_api_key.id
    firebase_auth_domain = google_secret_manager_secret.firebase_auth_domain.id
  }
  cicd_secret_ids = merge(local.backend_secret_ids, local.frontend_build_secret_ids)
}

resource "google_secret_manager_secret_iam_member" "backend_secrets" {
  for_each  = local.backend_secret_ids
  secret_id = each.value
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.backend_sa.email}"
}

resource "google_secret_manager_secret_iam_member" "cicd_secrets" {
  for_each  = local.cicd_secret_ids
  secret_id = each.value
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

# CICD Permissions
resource "google_artifact_registry_repository_iam_member" "repo_writer" {
  project    = var.project_id
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

resource "google_project_iam_member" "firebase_hosting_admin" {
  project = var.project_id
  role    = "roles/firebasehosting.admin"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_project_iam_member" "github_editor" {
  project = var.project_id
  role    = "roles/editor"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_project_iam_member" "github_project_iam_admin" {
  project = var.project_id
  role    = "roles/resourcemanager.projectIamAdmin"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_project_iam_member" "github_service_account_admin" {
  project = var.project_id
  role    = "roles/iam.serviceAccountAdmin"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_project_iam_member" "github_workload_identity_pool_admin" {
  project = var.project_id
  role    = "roles/iam.workloadIdentityPoolAdmin"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_project_iam_member" "firebase_api_keys_viewer" {
  project = var.project_id
  role    = "roles/serviceusage.apiKeysViewer"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

# Grant Firebase Auth access to the backend service account
resource "google_project_iam_member" "backend_firebase_auth" {
  project = var.project_id
  role    = "roles/firebaseauth.admin"
  member  = "serviceAccount:${google_service_account.backend_sa.email}"
}

output "backend_sa_email" {
  value = google_service_account.backend_sa.email
}

output "backend_sa_unique_id" {
  value = google_service_account.backend_sa.unique_id
}

output "github_actions_sa_email" {
  value = google_service_account.github_actions_sa.email
}

output "github_actions_sa_unique_id" {
  value = google_service_account.github_actions_sa.unique_id
}

resource "google_service_account_iam_member" "sa_user_backend" {
  service_account_id = google_service_account.backend_sa.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_service_account_iam_member" "backend_token_creator" {
  service_account_id = google_service_account.backend_sa.name
  role               = "roles/iam.serviceAccountTokenCreator"
  member             = "serviceAccount:${google_service_account.backend_sa.email}"
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
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repository}"
}
