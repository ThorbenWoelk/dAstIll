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

variable "ollama_url" {
  type        = string
  default     = "http://localhost:11434"
  description = "Ollama URL"
}

variable "ollama_model" {
  type        = string
  description = "Ollama model"
}

variable "summary_evaluator_model" {
  type        = string
  description = "Summary evaluator model"
}

variable "ollama_fallback_model" {
  type        = string
  description = "Ollama fallback model"
}

variable "ollama_embedding_model" {
  type        = string
  description = "Ollama embedding model"
}
