resource "google_cloud_run_v2_service" "backend" {
  provider            = google-beta
  name                = "${var.app_name}-backend"
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false

  template {
    service_account = google_service_account.backend_sa.email
    containers {
      image = "us-docker.pkg.dev/cloudrun/container/hello" # Placeholder, updated by CI/CD

      ports {
        container_port = 3001
      }


      env {
        name = "DB_URL"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.db_url.secret_id
            version = "latest"
          }
        }
      }

      env {
        name = "DB_PASS"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.db_pass.secret_id
            version = "latest"
          }
        }
      }

      env {
        name = "YOUTUBE_API_KEY"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.youtube_api_key.secret_id
            version = "latest"
          }
        }
      }

      env {
        name  = "OLLAMA_URL"
        value = var.ollama_url
      }

      env {
        name  = "OLLAMA_MODEL"
        value = var.ollama_model
      }

      env {
        name  = "SUMMARY_EVALUATOR_MODEL"
        value = var.summary_evaluator_model
      }

      env {
        name  = "RUST_LOG"
        value = "dastill=info,tower_http=info"
      }

      env {
        name  = "SUMMARIZE_PATH"
        value = "/usr/local/bin/summarize"
      }

      resources {
        cpu_idle          = true
        startup_cpu_boost = true
        limits = {
          cpu    = "1000m"
          memory = "512Mi"
        }
      }
    }
  }

  lifecycle {
    ignore_changes = [
      template[0].containers[0].image,
    ]
  }
}

resource "google_cloud_run_v2_service" "frontend" {
  provider            = google-beta
  name                = "${var.app_name}-frontend"
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false

  template {
    service_account = google_service_account.frontend_sa.email
    containers {
      image = "us-docker.pkg.dev/cloudrun/container/hello" # Placeholder, updated by CI/CD

      ports {
        container_port = 3000
      }


      # VITE_API_BASE will be set by CI/CD as an environment variable

      resources {
        cpu_idle          = true
        startup_cpu_boost = true
        limits = {
          cpu    = "1000m"
          memory = "512Mi"
        }
      }
    }
  }

  lifecycle {
    ignore_changes = [
      template[0].containers[0].image,
      template[0].containers[0].env,
    ]
  }
}

resource "google_cloud_run_v2_service_iam_member" "backend_public" {
  location = google_cloud_run_v2_service.backend.location
  name     = google_cloud_run_v2_service.backend.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

resource "google_cloud_run_v2_service_iam_member" "frontend_public" {
  location = google_cloud_run_v2_service.frontend.location
  name     = google_cloud_run_v2_service.frontend.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

output "backend_url" {
  value = google_cloud_run_v2_service.backend.uri
}

output "frontend_url" {
  value = google_cloud_run_v2_service.frontend.uri
}

resource "google_cloud_run_v2_service" "docs" {
  provider            = google-beta
  name                = "${var.app_name}-docs"
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false

  template {
    service_account = google_service_account.docs_sa.email
    containers {
      image = "us-docker.pkg.dev/cloudrun/container/hello" # Placeholder, updated by CI/CD

      ports {
        container_port = 8080
      }

      resources {
        cpu_idle          = true
        startup_cpu_boost = true
        limits = {
          cpu    = "1000m"
          memory = "512Mi"
        }
      }
    }
  }

  lifecycle {
    ignore_changes = [
      template[0].containers[0].image,
    ]
  }
}

resource "google_cloud_run_v2_service_iam_member" "docs_public" {
  location = google_cloud_run_v2_service.docs.location
  name     = google_cloud_run_v2_service.docs.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

output "docs_url" {
  value = google_cloud_run_v2_service.docs.uri
}
