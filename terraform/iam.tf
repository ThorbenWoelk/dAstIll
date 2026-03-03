resource "google_service_account" "backend_sa" {
  account_id   = "${var.app_name}-backend-sa"
  display_name = "${var.app_name} Backend Service Account"
}

resource "google_service_account" "frontend_sa" {
  account_id   = "${var.app_name}-frontend-sa"
  display_name = "${var.app_name} Frontend Service Account"
}

# Service Account for GitHub Actions
resource "google_service_account" "github_actions_sa" {
  account_id   = "${var.app_name}-github-sa"
  display_name = "${var.app_name} GitHub Actions Service Account"
}

# Grant access to secrets for backend
resource "google_secret_manager_secret_iam_member" "backend_secrets" {
  for_each = {
    db_url          = google_secret_manager_secret.db_url.id
    db_pass         = google_secret_manager_secret.db_pass.id
    youtube_api_key = google_secret_manager_secret.youtube_api_key.id
  }

  secret_id = each.value
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.backend_sa.email}"
}

# CICD Permissions
resource "google_artifact_registry_repository_iam_member" "repo_writer" {
  location   = google_artifact_registry_repository.repo.location
  repository = google_artifact_registry_repository.repo.name
  role       = "roles/artifactregistry.writer"
  member     = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_project_iam_member" "cloud_run_developer" {
  project = var.project_id
  role    = "roles/run.developer"
  member  = "serviceAccount:${google_service_account.github_actions_sa.email}"
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
  member             = "principalSet://iam.googleapis.com/projects/673062863574/locations/global/workloadIdentityPools/github-pool-v1/attribute.repository/ThorbenWoelk/dastill"
}
