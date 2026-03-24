# Spec: Migrate Video metadata from S3 to Firestore

## Problem

Marking a video as read while filtering by "unread" causes the video to reappear in the list. The root cause is S3's read-modify-write latency combined with three layers of caching (backend ReadCache 10s, frontend GET cache 30s, frontend channel view cache). A `load_all_videos` call lists all S3 keys then GETs each one in parallel - for a channel with 100 videos this takes 500ms-2s, during which stale `acknowledged` state can be served.

## Solution

Migrate Video metadata from S3 JSON files to Google Cloud Firestore. This gives:
- Strong read-after-write consistency (no eventual consistency issues)
- Atomic field updates (no read-modify-write cycle for `acknowledged`)
- Server-side filtering and ordering (no `load_all` + in-memory filter)
- Sub-10ms reads and writes (vs 50-200ms per S3 object)

## Scope

**Migrate to Firestore:**
- Video documents (the `Video` struct: id, channel_id, title, thumbnail_url, published_at, is_short, transcript_status, summary_status, acknowledged, retry_count, quality_score)

**Keep in S3:**
- Transcripts, summaries, video_info (large blobs, infrequently updated)
- Channels, highlights, chat conversations (separate concern, no consistency issues)
- Search vectors (S3 Vectors)

## Firestore Schema

Database: `(default)` in `europe-west3`

### Collection: `videos`

Document ID: YouTube video ID (e.g., `dQw4w9WgXcQ`)

```
{
  "channel_id": "UC...",
  "title": "Video Title",
  "thumbnail_url": "https://...",
  "published_at": Timestamp,
  "is_short": false,
  "transcript_status": "ready",
  "summary_status": "ready",
  "acknowledged": false,
  "retry_count": 0,
  "quality_score": 7
}
```

### Composite Indexes (minimal set)

1. `videos` - channel_id ASC, published_at DESC
   - Covers: list videos by channel sorted by date
   - With equality filter on channel_id + range/order on published_at

Firestore automatically handles single-field equality filters as extensions of composite indexes, so queries like:
- `WHERE channel_id == X AND acknowledged == false ORDER BY published_at DESC`
- `WHERE channel_id == X AND is_short == false ORDER BY published_at DESC`
- `WHERE channel_id == X AND is_short == false AND acknowledged == false ORDER BY published_at DESC`

All require additional composite indexes. To minimize index count on pay-as-you-go:

2. `videos` - channel_id ASC, acknowledged ASC, published_at DESC
3. `videos` - channel_id ASC, is_short ASC, published_at DESC
4. `videos` - channel_id ASC, is_short ASC, acknowledged ASC, published_at DESC

## Infrastructure Changes

### Terraform

1. Enable `firestore.googleapis.com` API
2. Create Firestore database `(default)` in `europe-west3` (FIRESTORE_NATIVE mode)
3. Grant `roles/datastore.user` to `dastill-backend-sa`
4. Create composite indexes via `google_firestore_index`

### CI/CD

- Pass `GCP_PROJECT_ID` env var to Cloud Run backend service
- No additional secrets needed (Firestore auth uses the Cloud Run service account identity)

## Backend Changes

### Dependencies

Add to Cargo.toml:
- `firestore = "0.46"` (abdolence/firestore-rs - Tokio-native, well-maintained)
- `gcloud-sdk` comes transitively

### Store Extension

Add `firestore: FirestoreDb` field to the `Store` struct. Initialize alongside S3 in main.rs.

### New Module: `db/firestore_videos.rs`

Replace all functions in `db/videos.rs` that do S3 operations with Firestore equivalents:
- `insert_video` - Firestore set (with merge for upsert)
- `get_video` - Firestore get document
- `list_videos_by_channel` - Firestore query with server-side filter/sort/pagination
- `update_video_acknowledged` - Firestore update single field (atomic, no read-modify-write)
- `update_video_transcript_status` - Firestore update single field
- `update_video_summary_status` - Firestore update single field
- `increment_video_retry_count` - Firestore increment
- `load_all_videos` - Only used by heal_queue and queue processing; use Firestore query
- `bulk_insert_videos` - Firestore batch write

### Cache Simplification

With Firestore's strong consistency:
- Backend ReadCache for channel snapshots can use shorter TTL or be removed for video queries
- Frontend can trust that a GET after a successful PUT returns fresh data
- The `videoListMutationEpoch` guard in the frontend becomes a safety net rather than essential

## Migration Strategy

1. Deploy Firestore infrastructure (Terraform)
2. Deploy backend with dual-write (S3 + Firestore) temporarily if needed
3. Backfill existing videos from S3 to Firestore (one-time script)
4. Switch reads to Firestore
5. Remove S3 video writes

For simplicity (single-user app), we can do a flag-day migration:
1. Deploy infrastructure
2. Run backfill script
3. Deploy new backend code that uses Firestore exclusively for videos

## Out of Scope

- Real-time Firestore listeners on the frontend (future enhancement)
- Migrating channels, transcripts, summaries, highlights, or chat to Firestore
- Offline support
