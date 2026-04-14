terraform {
  required_version = ">= 1.7.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "~> 6.0"
    }
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.24"
    }
    time = {
      source  = "hashicorp/time"
      version = "~> 0.13"
    }
  }

  backend "gcs" {
    # Bucket name will be provided via -backend-config
  }
}

provider "google" {
  billing_project       = var.project_id
  project               = var.project_id
  region                = var.region
  user_project_override = true
}

provider "google-beta" {
  billing_project       = var.project_id
  project               = var.project_id
  region                = var.region
  user_project_override = true
}

provider "aws" {
  region = var.aws_region
}
