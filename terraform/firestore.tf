resource "google_firestore_database" "default" {
  provider    = google-beta
  name        = "(default)"
  location_id = "nam5"
  type        = "FIRESTORE_NATIVE"

  depends_on = [google_project_service.services["firestore.googleapis.com"]]
}

# Composite index: list videos by channel, ordered by date
resource "google_firestore_index" "videos_channel_date" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Composite index: filter by channel + acknowledged, ordered by date
resource "google_firestore_index" "videos_channel_ack_date" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }
  fields {
    field_path = "acknowledged"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Composite index: filter by channel + is_short, ordered by date
resource "google_firestore_index" "videos_channel_short_date" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }
  fields {
    field_path = "is_short"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Composite index: filter by channel + is_short + acknowledged, ordered by date
resource "google_firestore_index" "videos_channel_short_ack_date" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }
  fields {
    field_path = "is_short"
    order      = "ASCENDING"
  }
  fields {
    field_path = "acknowledged"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

