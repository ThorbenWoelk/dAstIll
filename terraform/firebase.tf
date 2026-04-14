locals {
  firebase_docs_site_id = "${var.project_id}-docs"
  firebase_authorized_domains = distinct(
    concat(
      [
        "localhost",
        "${var.project_id}.firebaseapp.com",
        "${var.project_id}.web.app",
      ],
      var.firebase_authorized_domains_extra,
    ),
  )
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

resource "google_firebase_hosting_site" "docs" {
  provider = google-beta
  project  = var.project_id
  site_id  = local.firebase_docs_site_id
  app_id   = google_firebase_web_app.frontend.app_id

  depends_on = [google_firebase_project.default]
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

output "firebase_web_app_id" {
  value = google_firebase_web_app.frontend.app_id
}

output "firebase_docs_url" {
  value = google_firebase_hosting_site.docs.default_url
}
