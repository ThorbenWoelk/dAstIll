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
