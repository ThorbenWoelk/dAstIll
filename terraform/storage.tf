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

  # Deleted or overwritten objects stay recoverable for 30 days.
  # Search snapshots have no age rule: the backend deletes superseded ones
  # after each publish. An age rule here once deleted the live snapshot
  # after a quiet month and forced a slow full rebuild on every cold start.
  lifecycle_rule {
    action {
      type = "Delete"
    }

    condition {
      age        = 30
      with_state = "ARCHIVED"
    }
  }

  depends_on = [google_project_service.services]
}

output "gcs_data_bucket" {
  value = google_storage_bucket.data.name
}
