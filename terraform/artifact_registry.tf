resource "google_artifact_registry_repository" "repo" {
  project       = var.project_id
  location      = var.region
  repository_id = var.app_name
  description   = "Docker repository for ${var.app_name}"
  format        = "DOCKER"

  cleanup_policy_dry_run = false

  cleanup_policies {
    id     = "keep-latest-5"
    action = "KEEP"
    most_recent_versions {
      keep_count = 5
    }
  }

  cleanup_policies {
    id     = "delete-rest"
    action = "DELETE"
    condition {
      tag_state  = "ANY"
      older_than = "86400s"
    }
  }

  depends_on = [google_project_service.services]
}
