resource "google_storage_bucket" "data" {
  project                     = var.project_id
  name                        = "${var.app_name}-data-${var.region}"
  location                    = var.region
  storage_class               = "STANDARD"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = false

  labels = {
    app = var.app_name
  }

  versioning {
    enabled = true
  }

  lifecycle_rule {
    action {
      type = "Delete"
    }

    condition {
      age        = 30
      with_state = "ARCHIVED"
    }
  }

  lifecycle_rule {
    action {
      type = "Delete"
    }

    condition {
      age            = 30
      matches_prefix = ["runtime-cache/libsql/snapshots/"]
    }
  }

  depends_on = [google_project_service.services]
}

output "gcs_data_bucket" {
  value = google_storage_bucket.data.name
}
