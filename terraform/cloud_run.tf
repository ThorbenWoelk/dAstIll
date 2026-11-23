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
  count               = var.compute_enabled ? 1 : 0
  provider            = google-beta
  name                = "${var.app_name}-backend"
  project             = var.project_id
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

    scaling {
      max_instance_count = 1
    }

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

resource "google_cloud_run_v2_service" "asr" {
  count               = var.compute_enabled ? 1 : 0
  provider            = google-beta
  name                = "${var.app_name}-asr"
  project             = var.project_id
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

    scaling {
      max_instance_count = 1
    }

    containers {
      image = "us-docker.pkg.dev/cloudrun/container/hello"

      ports {
        container_port = 5092
      }

      resources {
        cpu_idle          = true
        startup_cpu_boost = true
        limits = {
          cpu    = "2000m"
          memory = "2048Mi"
        }
      }
    }
  }

  lifecycle {
    ignore_changes = [template, client, client_version]
  }
}

resource "google_cloud_run_v2_service_iam_member" "backend_public" {
  count    = var.compute_enabled ? 1 : 0
  project  = var.project_id
  location = google_cloud_run_v2_service.backend[0].location
  name     = google_cloud_run_v2_service.backend[0].name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

resource "google_cloud_run_v2_service_iam_member" "asr_backend_invoker" {
  count    = var.compute_enabled ? 1 : 0
  project  = var.project_id
  location = google_cloud_run_v2_service.asr[0].location
  name     = google_cloud_run_v2_service.asr[0].name
  role     = "roles/run.invoker"
  member   = "serviceAccount:${google_service_account.backend_sa.email}"
}

output "backend_url" {
  value = one(google_cloud_run_v2_service.backend[*].uri)
}

output "asr_url" {
  value = one(google_cloud_run_v2_service.asr[*].uri)
}

moved {
  from = google_cloud_run_v2_service.backend
  to   = google_cloud_run_v2_service.backend[0]
}

moved {
  from = google_cloud_run_v2_service.asr
  to   = google_cloud_run_v2_service.asr[0]
}

moved {
  from = google_cloud_run_v2_service_iam_member.backend_public
  to   = google_cloud_run_v2_service_iam_member.backend_public[0]
}

moved {
  from = google_cloud_run_v2_service_iam_member.asr_backend_invoker
  to   = google_cloud_run_v2_service_iam_member.asr_backend_invoker[0]
}
