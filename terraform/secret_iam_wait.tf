# Cloud Run validates Secret Manager access when creating a revision. Bindings can
# lag briefly after the IAM API returns success; without a pause, Terraform can
# still hit permission denied on the same apply.
resource "time_sleep" "after_backend_secret_accessor_bindings" {
  depends_on = [
    google_secret_manager_secret_iam_member.backend_secrets,
    google_secret_manager_secret_iam_member.backend_databricks_token,
  ]
  create_duration = "45s"
}
