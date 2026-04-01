locals {
  firebase_frontend_host = trimsuffix(
    trimprefix(google_cloud_run_v2_service.frontend.uri, "https://"),
    "/",
  )
  firebase_authorized_domains = distinct(
    concat(
      [
        "localhost",
        "${var.project_id}.firebaseapp.com",
        "${var.project_id}.web.app",
      ],
      compact([local.firebase_frontend_host]),
      var.firebase_authorized_domains_extra,
    ),
  )
  firebase_google_provider_enabled = trimspace(var.firebase_google_client_id) != "" && trimspace(var.firebase_google_client_secret) != ""
}

resource "google_firebase_project" "default" {
  provider = google-beta
  project  = var.project_id

  depends_on = [google_project_service.services["firebase.googleapis.com"]]
}

resource "google_firebase_web_app" "frontend" {
  provider     = google-beta
  project      = var.project_id
  display_name = var.firebase_web_app_display_name

  depends_on = [google_firebase_project.default]
}

data "google_firebase_web_app_config" "frontend" {
  provider   = google-beta
  project    = var.project_id
  web_app_id = google_firebase_web_app.frontend.app_id

  depends_on = [google_firebase_web_app.frontend]
}

resource "google_identity_platform_config" "default" {
  provider           = google-beta
  project            = var.project_id
  authorized_domains = local.firebase_authorized_domains

  sign_in {
    anonymous {
      enabled = true
    }
  }

  depends_on = [google_firebase_project.default]
}

resource "google_identity_platform_default_supported_idp_config" "google" {
  count         = local.firebase_google_provider_enabled ? 1 : 0
  provider      = google-beta
  project       = var.project_id
  idp_id        = "google.com"
  enabled       = true
  client_id     = var.firebase_google_client_id
  client_secret = var.firebase_google_client_secret

  depends_on = [google_identity_platform_config.default]
}

output "firebase_web_app_id" {
  value = google_firebase_web_app.frontend.app_id
}

output "firebase_auth_domain" {
  value = data.google_firebase_web_app_config.frontend.auth_domain
}
