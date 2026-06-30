locals {
  billing_budget_project_ids          = setunion(toset([var.project_id]), var.billing_budget_project_ids)
  billing_budget_cloud_run_service_id = "services/152E-C115-5142"
  billing_budget_cloud_storage_id     = "services/95FF-2EF5-5EA1"
}

data "google_project" "billing_budget_projects" {
  for_each = var.billing_budgets_enabled ? local.billing_budget_project_ids : toset([])

  project_id = each.key
}

locals {
  billing_budget_project_numbers = {
    for project_id, project in data.google_project.billing_budget_projects :
    project_id => project.number
  }

  billing_budget_project_account_ids = {
    for project_id, project in data.google_project.billing_budget_projects :
    project_id => replace(
      trimspace(lookup(var.billing_budget_project_billing_account_ids, project_id, var.billing_budget_billing_account_id)),
      "billingAccounts/",
      ""
    )
  }
}

resource "google_billing_budget" "dastill_project" {
  for_each = var.billing_budgets_enabled ? local.billing_budget_project_ids : toset([])

  billing_account = local.billing_budget_project_account_ids[each.key]
  display_name    = "${var.app_name} ${each.key} monthly"

  budget_filter {
    projects               = ["projects/${local.billing_budget_project_numbers[each.key]}"]
    calendar_period        = "MONTH"
    credit_types_treatment = "INCLUDE_ALL_CREDITS"
  }

  amount {
    specified_amount {
      units = var.billing_budget_app_monthly_amount_units
    }
  }

  dynamic "threshold_rules" {
    for_each = var.billing_budget_thresholds

    content {
      threshold_percent = threshold_rules.value.threshold_percent
      spend_basis       = threshold_rules.value.spend_basis
    }
  }

  lifecycle {
    precondition {
      condition     = local.billing_budget_project_account_ids[each.key] != ""
      error_message = "Set billing_budget_billing_account_id or billing_budget_project_billing_account_ids[\"${each.key}\"] before enabling billing budgets."
    }
  }

  depends_on = [google_project_service.services]
}

resource "google_billing_budget" "cloud_run" {
  for_each = var.billing_budgets_enabled ? local.billing_budget_project_ids : toset([])

  billing_account = local.billing_budget_project_account_ids[each.key]
  display_name    = "${var.app_name} Cloud Run ${each.key} monthly"

  budget_filter {
    projects               = ["projects/${local.billing_budget_project_numbers[each.key]}"]
    services               = [local.billing_budget_cloud_run_service_id]
    calendar_period        = "MONTH"
    credit_types_treatment = "INCLUDE_ALL_CREDITS"
  }

  amount {
    specified_amount {
      units = var.billing_budget_cloud_run_monthly_amount_units
    }
  }

  dynamic "threshold_rules" {
    for_each = var.billing_budget_thresholds

    content {
      threshold_percent = threshold_rules.value.threshold_percent
      spend_basis       = threshold_rules.value.spend_basis
    }
  }

  lifecycle {
    precondition {
      condition     = local.billing_budget_project_account_ids[each.key] != ""
      error_message = "Set billing_budget_billing_account_id or billing_budget_project_billing_account_ids[\"${each.key}\"] before enabling billing budgets."
    }
  }

  depends_on = [google_project_service.services]
}

resource "google_billing_budget" "cloud_storage" {
  for_each = var.billing_budgets_enabled ? local.billing_budget_project_ids : toset([])

  billing_account = local.billing_budget_project_account_ids[each.key]
  display_name    = "${var.app_name} Cloud Storage ${each.key} monthly"

  budget_filter {
    projects               = ["projects/${local.billing_budget_project_numbers[each.key]}"]
    services               = [local.billing_budget_cloud_storage_id]
    calendar_period        = "MONTH"
    credit_types_treatment = "INCLUDE_ALL_CREDITS"
  }

  amount {
    specified_amount {
      units = var.billing_budget_cloud_storage_monthly_amount_units
    }
  }

  dynamic "threshold_rules" {
    for_each = var.billing_budget_thresholds

    content {
      threshold_percent = threshold_rules.value.threshold_percent
      spend_basis       = threshold_rules.value.spend_basis
    }
  }

  lifecycle {
    precondition {
      condition     = local.billing_budget_project_account_ids[each.key] != ""
      error_message = "Set billing_budget_billing_account_id or billing_budget_project_billing_account_ids[\"${each.key}\"] before enabling billing budgets."
    }
  }

  depends_on = [google_project_service.services]
}
