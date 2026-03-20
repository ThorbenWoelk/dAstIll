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

resource "google_secret_manager_secret" "app_auth_password" {
  secret_id = "${var.app_name}-app-auth-password"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "app_auth_password" {
  secret      = google_secret_manager_secret.app_auth_password.id
  secret_data = var.app_auth_password
}

resource "google_secret_manager_secret" "app_session_secret" {
  secret_id = "${var.app_name}-app-session-secret"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "app_session_secret" {
  secret      = google_secret_manager_secret.app_session_secret.id
  secret_data = var.app_session_secret
}

resource "google_secret_manager_secret" "backend_proxy_token" {
  secret_id = "${var.app_name}-backend-proxy-token"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "backend_proxy_token" {
  secret      = google_secret_manager_secret.backend_proxy_token.id
  secret_data = var.backend_proxy_token
}
