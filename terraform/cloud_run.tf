resource "google_cloud_run_v2_service" "backend" {
  provider            = google-beta
  name                = "${var.app_name}-backend"
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false

  template {
    service_account = google_service_account.backend_sa.email
    containers {
      image = "us-docker.pkg.dev/cloudrun/container/hello"

      ports {
        container_port = 3001
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
    ignore_changes = [template]
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
      image = "us-docker.pkg.dev/cloudrun/container/hello"

      ports {
        container_port = 3000
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
    ignore_changes = [template]
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
      image = "us-docker.pkg.dev/cloudrun/container/hello"

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
    ignore_changes = [template]
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
