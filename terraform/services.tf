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
      "storage.googleapis.com",
      "storagetransfer.googleapis.com",
      "texttospeech.googleapis.com",
    ],
    var.billing_export_enabled ? [
      "bigquery.googleapis.com",
      "bigquerydatatransfer.googleapis.com",
    ] : [],
    var.billing_budgets_enabled ? [
      "billingbudgets.googleapis.com",
    ] : [],
  )
}

resource "google_project_service" "services" {
  for_each = toset(local.project_services)

  project            = var.project_id
  service            = each.key
  disable_on_destroy = false
}
