# Scheduled background processing.
#
# The backend only gets CPU while it serves a request, and its workers pause
# while no user is active. Without readers, channels are never refreshed and
# new videos are never summarized. This job wakes the backend every 4 hours:
# the catch-up endpoint refreshes all channels and works through pending
# transcripts and summaries for up to 10 minutes.
#
# The job signs an OIDC token as its own service account. The backend accepts
# only that account and audience (CATCH_UP_INVOKER_EMAIL and
# CATCH_UP_AUDIENCE, set in .github/workflows/deploy.yml). No secret is stored.

locals {
  catch_up_audience = "${var.app_name}-catch-up"
}

resource "google_service_account" "scheduler_sa" {
  project      = var.project_id
  account_id   = "${var.app_name}-scheduler-sa"
  display_name = "${var.app_name} Cloud Scheduler Service Account"
}

# Terraform (the GitHub Actions account) must be allowed to attach the
# scheduler account to the job.
resource "google_service_account_iam_member" "sa_user_scheduler" {
  service_account_id = google_service_account.scheduler_sa.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.github_actions_sa.email}"
}

resource "google_cloud_scheduler_job" "catch_up" {
  project          = var.project_id
  region           = var.region
  name             = "${var.app_name}-catch-up"
  description      = "Refresh channels and process pending summaries."
  schedule         = "0 */4 * * *"
  time_zone        = "Etc/UTC"
  attempt_deadline = "900s"

  retry_config {
    retry_count = 0
  }

  http_target {
    http_method = "POST"
    uri         = "${google_cloud_run_v2_service.backend.uri}/api/internal/catch-up"

    oidc_token {
      service_account_email = google_service_account.scheduler_sa.email
      audience              = local.catch_up_audience
    }
  }

  depends_on = [
    google_project_service.services,
    google_service_account_iam_member.sa_user_scheduler,
  ]
}

output "scheduler_sa_email" {
  value = google_service_account.scheduler_sa.email
}
