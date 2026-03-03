resource "google_service_account" "backend_sa" {
  account_id   = "${var.app_name}-backend-sa"
  display_name = "${var.app_name} Backend Service Account"
}

resource "google_service_account" "frontend_sa" {
  account_id   = "${var.app_name}-frontend-sa"
  display_name = "${var.app_name} Frontend Service Account"
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
