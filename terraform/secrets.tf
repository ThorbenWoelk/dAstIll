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

resource "google_secret_manager_secret" "databricks_token" {
  count     = var.databricks_token != "" ? 1 : 0
  secret_id = "${var.app_name}-databricks-token"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "databricks_token" {
  count       = var.databricks_token != "" ? 1 : 0
  secret      = google_secret_manager_secret.databricks_token[0].id
  secret_data = var.databricks_token
}

data "google_secret_manager_secret" "databricks_token" {
  count     = var.databricks_token == "" ? 1 : 0
  secret_id = "${var.app_name}-databricks-token"
}

locals {
  firebase_auth_domain_effective = trimspace(var.firebase_auth_domain) != "" ? trimspace(var.firebase_auth_domain) : "${var.project_id}.firebaseapp.com"
  firebase_secrets_enabled = nonsensitive(length(trimspace(var.firebase_web_api_key)) > 0)
}

resource "google_secret_manager_secret" "firebase_web_api_key" {
  count     = local.firebase_secrets_enabled ? 1 : 0
  secret_id = "${var.app_name}-firebase-web-api-key"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "firebase_web_api_key" {
  count       = local.firebase_secrets_enabled ? 1 : 0
  secret      = google_secret_manager_secret.firebase_web_api_key[0].id
  secret_data = var.firebase_web_api_key
}

resource "google_secret_manager_secret" "firebase_auth_domain" {
  count     = local.firebase_secrets_enabled ? 1 : 0
  secret_id = "${var.app_name}-firebase-auth-domain"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "firebase_auth_domain" {
  count       = local.firebase_secrets_enabled ? 1 : 0
  secret      = google_secret_manager_secret.firebase_auth_domain[0].id
  secret_data = local.firebase_auth_domain_effective
}
