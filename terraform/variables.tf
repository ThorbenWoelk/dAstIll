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

variable "app_auth_password" {
  type        = string
  sensitive   = true
  description = "Frontend operator password"
}

variable "app_session_secret" {
  type        = string
  sensitive   = true
  description = "Frontend session signing secret"
}

variable "backend_proxy_token" {
  type        = string
  sensitive   = true
  description = "Shared proxy secret for frontend-to-backend requests"
}
