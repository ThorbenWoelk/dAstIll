locals {
  billing_export_project_id = trimspace(var.billing_export_project_id) != "" ? trimspace(var.billing_export_project_id) : var.project_id
}

resource "google_bigquery_dataset" "billing_export" {
  count = var.billing_export_enabled ? 1 : 0

  project    = local.billing_export_project_id
  dataset_id = var.billing_export_dataset_id
  location   = var.billing_export_dataset_location

  friendly_name = "Cloud Billing Export"
  description   = "Cloud Billing usage and pricing export dataset for ${var.project_id}."

  labels = {
    app         = var.app_name
    managed_by  = "terraform"
    usage_scope = "billing_export"
  }
}

resource "google_bigquery_dataset_access" "billing_export_usage_writer" {
  count = var.billing_export_enabled ? 1 : 0

  project    = google_bigquery_dataset.billing_export[0].project
  dataset_id = google_bigquery_dataset.billing_export[0].dataset_id
  role       = "OWNER"
  iam_member = "serviceAccount:billing-export-bigquery@system.gserviceaccount.com"
}

resource "google_bigquery_dataset_access" "billing_export_pricing_writer" {
  count = var.billing_export_enabled ? 1 : 0

  project    = google_bigquery_dataset.billing_export[0].project
  dataset_id = google_bigquery_dataset.billing_export[0].dataset_id
  role       = "OWNER"
  iam_member = "serviceAccount:cloud-account-pricing@cloud-account-pricing.iam.gserviceaccount.com"
}
