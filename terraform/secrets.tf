resource "google_secret_manager_secret" "db_url" {
  secret_id = "${var.app_name}-db-url"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "db_url" {
  secret      = google_secret_manager_secret.db_url.id
  secret_data = var.db_url
}

resource "google_secret_manager_secret" "db_pass" {
  secret_id = "${var.app_name}-db-pass"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "db_pass" {
  secret      = google_secret_manager_secret.db_pass.id
  secret_data = var.db_pass
}

resource "google_secret_manager_secret" "youtube_api_key" {
  secret_id = "${var.app_name}-youtube-api-key"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "youtube_api_key" {
  secret      = google_secret_manager_secret.youtube_api_key.id
  secret_data = var.youtube_api_key
}
