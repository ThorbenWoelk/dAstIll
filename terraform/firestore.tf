resource "google_firestore_database" "default" {
  provider    = google-beta
  name        = "(default)"
  location_id = "nam5"
  type        = "FIRESTORE_NATIVE"

  depends_on = [google_project_service.services["firestore.googleapis.com"]]
}

# Composite index: list videos by channel with ready transcripts, ordered by date
resource "google_firestore_index" "videos_channel_date" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }
  fields {
    field_path = "transcript_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Composite index: filter by channel + acknowledged + ready transcript, ordered by date
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
    field_path = "transcript_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Composite index: filter by channel + is_short + ready transcript, ordered by date
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
    field_path = "transcript_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Composite index: filter by channel + is_short + acknowledged + ready transcript, ordered by date
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
    field_path = "transcript_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Composite index: find oldest fully ready video by channel
resource "google_firestore_index" "videos_channel_fully_ready_oldest" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "channel_id"
    order      = "ASCENDING"
  }
  fields {
    field_path = "transcript_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "summary_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "ASCENDING"
  }
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

# Exact composite index requested by Firestore for ready-transcript channel list/snapshot queries
resource "google_firestore_index" "videos_ready_transcript_published_desc" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "transcript_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "DESCENDING"
  }
}

# Exact composite index requested by Firestore for oldest fully-ready lookups
resource "google_firestore_index" "videos_ready_summary_published_asc" {
  provider   = google-beta
  database   = google_firestore_database.default.name
  collection = "dastill_videos"

  fields {
    field_path = "summary_status"
    order      = "ASCENDING"
  }
  fields {
    field_path = "published_at"
    order      = "ASCENDING"
  }
}
