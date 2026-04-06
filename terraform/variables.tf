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
  description = "Extra Firebase Auth authorized domains to keep in addition to localhost, firebaseapp/web.app, and the Cloud Run frontend hostname."
}

variable "aws_region" {
  type        = string
  default     = "eu-central-1"
  description = "AWS region for S3 and S3 Vectors"
}



variable "youtube_api_key" {
  type        = string
  sensitive   = true
  description = "YouTube API key"
}

variable "ollama_api_key" {
  type        = string
  sensitive   = true
  description = "Ollama API key for authenticated cloud endpoints"
}

variable "logfire_token" {
  type        = string
  sensitive   = true
  description = "Logfire token for production telemetry"
}

variable "backend_proxy_token" {
  type        = string
  sensitive   = true
  description = "Shared proxy secret for frontend-to-backend requests"
}

variable "turso_auth_token" {
  type        = string
  sensitive   = true
  default     = ""
  description = "Turso auth token for the backend database (videos, preferences, TTS stats, keyword search). When set, Terraform writes it to Secret Manager as <app_name>-turso-auth-token."
}

variable "databricks_token" {
  type        = string
  sensitive   = true
  default     = ""
  description = "Databricks PAT. If non-empty, Terraform manages the secret version in Secret Manager. If empty, only IAM is managed and the databricks-token secret must already exist."
}

variable "firebase_web_api_key" {
  type        = string
  sensitive   = true
  default     = ""
  description = "Firebase Web API key (Project settings > General). If non-empty, Terraform creates Secret Manager secrets for the web client. If empty, omit firebase_* from terraform.tfvars until ready."
}

variable "firebase_auth_domain" {
  type        = string
  default     = ""
  description = "Firebase authDomain (e.g. project.firebaseapp.com). Leave empty to use {project_id}.firebaseapp.com."
}

variable "firebase_google_client_id" {
  type        = string
  sensitive   = true
  default     = ""
  description = "Deprecated. Ignored. Google sign-in is managed via frontend/firebase.json and a separate one-time or maintenance auth deploy."
}

variable "firebase_google_client_secret" {
  type        = string
  sensitive   = true
  default     = ""
  description = "Deprecated. Ignored. Google sign-in is managed via frontend/firebase.json and a separate one-time or maintenance auth deploy."
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
