resource "google_firestore_database" "default" {
  provider    = google-beta
  name        = "(default)"
  location_id = var.firestore_location_id
  project     = var.project_id
  type        = "FIRESTORE_NATIVE"

  depends_on = [google_project_service.services["firestore.googleapis.com"]]
}

# Single-field video index policy: allow only fields still used by equality queries and
# turn the rest off. Ordered per-channel reads use explicit composite indexes below.
resource "google_firestore_field" "videos_id_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "id"

  index_config {}
}

resource "google_firestore_field" "videos_channel_id_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "channel_id"

  index_config {}
}

resource "google_firestore_field" "videos_title_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "title"

  index_config {} # Empty index_config disables all single-field indexes for this field
}

resource "google_firestore_field" "videos_thumbnail_url_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "thumbnail_url"

  index_config {}
}

resource "google_firestore_field" "videos_quality_score_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "quality_score"

  index_config {}
}

resource "google_firestore_field" "videos_retry_count_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "retry_count"

  index_config {}
}

resource "google_firestore_field" "videos_published_at_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "published_at"

  index_config {}
}

resource "google_firestore_field" "videos_is_short_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "is_short"

  index_config {}
}

resource "google_firestore_field" "videos_acknowledged_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "acknowledged"

  index_config {}
}

resource "google_firestore_field" "videos_transcript_status_index" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "transcript_status"

  index_config {
    indexes {
      order       = "ASCENDING"
      query_scope = "COLLECTION"
    }
  }
}

resource "google_firestore_field" "videos_summary_status_index" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "summary_status"

  index_config {
    indexes {
      order       = "ASCENDING"
      query_scope = "COLLECTION"
    }
  }
}

resource "google_firestore_index" "videos_by_channel_published_at_desc" {
  provider    = google-beta
  project     = var.project_id
  database    = google_firestore_database.default.name
  collection  = "dastill_videos"
  query_scope = "COLLECTION"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }

  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

resource "google_firestore_index" "videos_by_channel_published_at_asc" {
  provider    = google-beta
  project     = var.project_id
  database    = google_firestore_database.default.name
  collection  = "dastill_videos"
  query_scope = "COLLECTION"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }

  fields {
    field_path = "published_at"
    order      = "ASCENDING"
  }
}

resource "google_firestore_field" "preferences_vocabulary_replacements_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_preferences"
  field      = "vocabulary_replacements"

  index_config {}
}

resource "google_firestore_field" "preferences_channel_order_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_preferences"
  field      = "channel_order"

  index_config {}
}

resource "google_firestore_field" "preferences_channel_sort_mode_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_preferences"
  field      = "channel_sort_mode"

  index_config {}
}

resource "google_firestore_field" "tts_stats_sample_count_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_tts_stats"
  field      = "sample_count"

  index_config {}
}

resource "google_firestore_field" "tts_stats_total_words_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_tts_stats"
  field      = "total_words"

  index_config {}
}

resource "google_firestore_field" "tts_stats_total_duration_secs_exemption" {
  provider   = google-beta
  project    = var.project_id
  database   = google_firestore_database.default.name
  collection = "dastill_tts_stats"
  field      = "total_duration_secs"

  index_config {}
}
