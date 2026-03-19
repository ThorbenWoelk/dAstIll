resource "google_secret_manager_secret" "ollama_api_key" {
  secret_id = "${var.app_name}-ollama-api-key"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "ollama_api_key" {
  secret      = google_secret_manager_secret.ollama_api_key.id
  secret_data = var.ollama_api_key
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

resource "google_secret_manager_secret" "logfire_token" {
  secret_id = "${var.app_name}-logfire-token"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "logfire_token" {
  secret      = google_secret_manager_secret.logfire_token.id
  secret_data = var.logfire_token
}
