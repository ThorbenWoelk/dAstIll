resource "google_firestore_database" "default" {
  provider    = google-beta
  name        = "(default)"
  location_id = "nam5"
  type        = "FIRESTORE_NATIVE"

  depends_on = [google_project_service.services["firestore.googleapis.com"]]
}

# Single-field index exemptions for unqueried fields to save storage and write costs
resource "google_firestore_field" "videos_title_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "title"

  index_config {} # Empty index_config disables all single-field indexes for this field
}

resource "google_firestore_field" "videos_thumbnail_url_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "thumbnail_url"

  index_config {}
}

resource "google_firestore_field" "videos_quality_score_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "quality_score"

  index_config {}
}

resource "google_firestore_field" "videos_retry_count_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"
  field      = "retry_count"

  index_config {}
}

resource "google_firestore_field" "preferences_vocabulary_replacements_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_preferences"
  field      = "vocabulary_replacements"

  index_config {}
}

resource "google_firestore_field" "preferences_channel_order_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_preferences"
  field      = "channel_order"

  index_config {}
}

resource "google_firestore_field" "preferences_channel_sort_mode_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_preferences"
  field      = "channel_sort_mode"

  index_config {}
}

resource "google_firestore_field" "tts_stats_sample_count_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_tts_stats"
  field      = "sample_count"

  index_config {}
}

resource "google_firestore_field" "tts_stats_total_words_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_tts_stats"
  field      = "total_words"

  index_config {}
}

resource "google_firestore_field" "tts_stats_total_duration_secs_exemption" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_tts_stats"
  field      = "total_duration_secs"

  index_config {}
}
