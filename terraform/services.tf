locals {
  project_services = concat(
    [
      "run.googleapis.com",
      "artifactregistry.googleapis.com",
      "secretmanager.googleapis.com",
      "firebase.googleapis.com",
      "firebasehosting.googleapis.com",
      "iam.googleapis.com",
      "iamcredentials.googleapis.com",
      "sts.googleapis.com",
      "cloudresourcemanager.googleapis.com",
      "identitytoolkit.googleapis.com",
    ],
    var.billing_export_enabled ? [
      "bigquery.googleapis.com",
      "bigquerydatatransfer.googleapis.com",
    ] : [],
  )
}

resource "google_project_service" "services" {
  for_each = toset(local.project_services)

  service            = each.key
  disable_on_destroy = false
}
