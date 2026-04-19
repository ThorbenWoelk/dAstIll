resource "google_secret_manager_secret" "ollama_api_key" {
  project   = var.project_id
  secret_id = "${var.app_name}-ollama-api-key"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret" "youtube_api_key" {
  project   = var.project_id
  secret_id = "${var.app_name}-youtube-api-key"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret" "openalex_api_key" {
  project   = var.project_id
  secret_id = "${var.app_name}-openalex-api-key"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret" "logfire_token" {
  project   = var.project_id
  secret_id = "${var.app_name}-logfire-token"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret" "backend_proxy_token" {
  project   = var.project_id
  secret_id = "${var.app_name}-backend-proxy-token"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret" "databricks_token" {
  project   = var.project_id
  secret_id = "${var.app_name}-databricks-token"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret" "firebase_web_api_key" {
  project   = var.project_id
  secret_id = "${var.app_name}-firebase-web-api-key"
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret" "firebase_auth_domain" {
  project   = var.project_id
  secret_id = "${var.app_name}-firebase-auth-domain"
  replication {
    auto {}
  }
}

removed {
  from = google_secret_manager_secret_version.ollama_api_key

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret_version.youtube_api_key

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret_version.openalex_api_key

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret_version.logfire_token

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret_version.backend_proxy_token

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret_version.databricks_token

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret.local_asr_api_key

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret_version.firebase_web_api_key

  lifecycle {
    destroy = false
  }
}

removed {
  from = google_secret_manager_secret_version.firebase_auth_domain

  lifecycle {
    destroy = false
  }
}
