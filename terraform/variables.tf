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

variable "billing_budgets_enabled" {
  type        = bool
  default     = false
  description = "When true, Terraform creates monthly Cloud Billing budgets for the app project and Cloud Run spend."
}

variable "billing_budget_project_ids" {
  type        = set(string)
  default     = []
  description = "Additional GCP project IDs with dAstIll Cloud Run deployments. The primary project_id is always included."

  validation {
    condition     = alltrue([for project_id in var.billing_budget_project_ids : trimspace(project_id) != ""])
    error_message = "billing_budget_project_ids cannot contain empty project IDs."
  }
}

variable "billing_budget_billing_account_id" {
  type        = string
  default     = ""
  description = "Billing account ID for all budgets. infra.yml resolves this from the primary project when unset in CI."
}

variable "billing_budget_project_billing_account_ids" {
  type        = map(string)
  default     = {}
  description = "Optional per-project billing account IDs for budget projects that use a different billing account than billing_budget_billing_account_id."
}

variable "billing_budget_app_monthly_amount_units" {
  type        = string
  default     = "50"
  description = "Whole-unit monthly budget amount for all dAstIll project spend, in the billing account currency."
}

variable "billing_budget_cloud_run_monthly_amount_units" {
  type        = string
  default     = "10"
  description = "Whole-unit monthly budget amount for Cloud Run spend per configured project, in the billing account currency."
}

variable "billing_budget_thresholds" {
  type = list(object({
    threshold_percent = number
    spend_basis       = optional(string, "CURRENT_SPEND")
  }))
  default = [
    {
      threshold_percent = 0.5
      spend_basis       = "CURRENT_SPEND"
    },
    {
      threshold_percent = 0.8
      spend_basis       = "CURRENT_SPEND"
    },
    {
      threshold_percent = 1.0
      spend_basis       = "CURRENT_SPEND"
    },
    {
      threshold_percent = 1.0
      spend_basis       = "FORECASTED_SPEND"
    },
  ]
  description = "Alert thresholds for all billing budgets. Percent values are 1.0-based, so 0.5 means 50%."

  validation {
    condition = alltrue([
      for threshold in var.billing_budget_thresholds :
      threshold.threshold_percent > 0 && contains(["CURRENT_SPEND", "FORECASTED_SPEND"], threshold.spend_basis)
    ])
    error_message = "Each billing budget threshold must have a positive threshold_percent and spend_basis CURRENT_SPEND or FORECASTED_SPEND."
  }
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
