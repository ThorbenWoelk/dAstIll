variable "project_id" {
  type        = string
  description = "The GCP project ID"
}

variable "region" {
  type        = string
  default     = "europe-west3"
  description = "The GCP region"
}

variable "app_name" {
  type        = string
  default     = "dastill"
  description = "The application name"
}

variable "firebase_web_app_display_name" {
  type        = string
  default     = "dAstIll Web"
  description = "Display name for the Firebase web app used by the product frontend."
}

variable "firebase_authorized_domains_extra" {
  type        = list(string)
  default     = []
  description = "Extra Firebase Auth authorized domains to keep in addition to localhost and the Firebase-hosted project domains."
}

variable "aws_region" {
  type        = string
  default     = "eu-central-1"
  description = "AWS region for S3 and S3 Vectors"
}
variable "firebase_google_client_id" {
  type        = string
  sensitive   = true
  default     = ""
  description = "Deprecated. Ignored. Google sign-in is managed via the repo-root firebase.json and a separate one-time or maintenance auth deploy."
}

variable "firebase_google_client_secret" {
  type        = string
  sensitive   = true
  default     = ""
  description = "Deprecated. Ignored. Google sign-in is managed via the repo-root firebase.json and a separate one-time or maintenance auth deploy."
}

variable "billing_export_enabled" {
  type        = bool
  default     = false
  description = "When true, Terraform creates the BigQuery dataset and API prerequisites for Cloud Billing export."
}

variable "billing_export_project_id" {
  type        = string
  default     = ""
  description = "Optional project ID that will host the billing export dataset. Defaults to project_id."
}

variable "billing_export_dataset_id" {
  type        = string
  default     = "billing_export"
  description = "BigQuery dataset ID for Cloud Billing export tables."
}

variable "billing_export_dataset_location" {
  type        = string
  default     = "EU"
  description = "BigQuery dataset location for Cloud Billing export tables."
}

variable "github_repository" {
  type        = string
  default     = "ThorbenWoelk/dAstIll"
  description = "GitHub repository slug allowed to impersonate the deploy service account through Workload Identity Federation."
}

variable "github_wif_pool_id" {
  type        = string
  default     = "github-pool-v1"
  description = "Workload Identity Pool ID that contains the GitHub provider."
}

variable "github_wif_provider_id" {
  type        = string
  default     = "github-provider-v1"
  description = "Workload Identity Provider ID used by GitHub Actions."
}
