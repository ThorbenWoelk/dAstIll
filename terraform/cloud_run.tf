locals {
  # Cloud Run's API always returns a service-level scaling block with zero counts.
  # Omitting `scaling` causes a perpetual plan (Terraform tries to drop it each run).
  # Do not set scaling_mode here: the provider often leaves it unset in state, which
  # would otherwise cause a perpetual diff against "AUTOMATIC".
  cloud_run_default_scaling = {
    min_instance_count    = 0
    manual_instance_count = 0
  }
}

resource "google_cloud_run_v2_service" "backend" {
  provider            = google-beta
  name                = "${var.app_name}-backend"
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false

  scaling {
    min_instance_count    = local.cloud_run_default_scaling.min_instance_count
    manual_instance_count = local.cloud_run_default_scaling.manual_instance_count
  }

  depends_on = [time_sleep.after_backend_secret_accessor_bindings]

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
          memory = "1024Mi"
        }
      }
    }
  }

  lifecycle {
    ignore_changes = [template, client, client_version]
  }
}

resource "google_cloud_run_v2_service" "frontend" {
  provider            = google-beta
  name                = "${var.app_name}-frontend"
  location            = var.region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = false

  scaling {
    min_instance_count    = local.cloud_run_default_scaling.min_instance_count
    manual_instance_count = local.cloud_run_default_scaling.manual_instance_count
  }

  depends_on = [
    google_secret_manager_secret_iam_member.frontend_secrets,
  ]

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
          memory = "256Mi"
        }
      }
    }
  }

  lifecycle {
    ignore_changes = [template, client, client_version]
  }
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

  scaling {
    min_instance_count    = local.cloud_run_default_scaling.min_instance_count
    manual_instance_count = local.cloud_run_default_scaling.manual_instance_count
  }

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
          memory = "256Mi"
        }
      }
    }
  }

  lifecycle {
    ignore_changes = [template, client, client_version]
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
