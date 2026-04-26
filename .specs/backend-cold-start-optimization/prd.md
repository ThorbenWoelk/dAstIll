# PRD: Backend Cold-Start Optimization

## Problem

Production `dastill-mini` can take more than 115 seconds to deliver first backend content after the Cloud Run backend scales from zero. The observed cold path is dominated by backend startup work, not Firebase Hosting.

The backend currently rebuilds local libSQL state from S3 before serving. In the measured cold start, SQL reconciliation bootstrapped 3004 videos. The current implementation can issue thousands of S3 object reads, thousands of transcript/summary existence checks, thousands of individual libSQL writes, and thousands of startup logs before Axum begins serving.

This makes the lightweight `/mini` fallback feel unavailable exactly when it should be most reliable.

## Goal

Reduce user-visible backend cold-start time for `dastill-mini` and the workspace first-data path, while preserving the current production constraint that the backend runs as one Cloud Run instance with local libSQL state and in-process workers.

## Current Increment

**Phase 1: Restore local libSQL from an S3 runtime-cache snapshot**

Implement the smallest safe increment that:

- Restores a compressed libSQL file from `runtime-cache/libsql/` before opening the local database.
- Validates the manifest schema version, snapshot checksum, and canonical S3 source-prefix fingerprints before accepting a snapshot.
- Falls back to the existing canonical S3 rebuild path when the snapshot is missing, stale, corrupt, or incompatible.
- Publishes a fresh derived snapshot after fallback rebuild or reconciliation changes.
- Leaves canonical S3 objects as the source of truth.

## Clear Deliverable

A cold backend instance can restore local libSQL from one compressed S3 snapshot when canonical source prefixes are unchanged, and can safely fall back to the existing S3 rebuild path when they changed. The implementation should show a material reduction in cold startup time on the second cold start after a valid snapshot is published, with no change to user-visible catalog semantics.

## Non-Goals

- Do not remove S3 canonical video snapshots.
- Do not introduce horizontal backend scale-out.
- Do not split workers from serving in this increment.
- Do not switch the durable catalog store away from S3.
- Do not make the runtime-cache snapshot the source of truth.
- Do not change `/mini` UX in this increment.

## Users Or Actors

- Signed-in users opening `dastill-mini` after the backend has scaled to zero.
- Operators watching release health and startup latency.
- Future maintainers debugging S3, libSQL, and Cloud Run startup behavior.

## Requirements

### Observability

- Startup logs must report total SQL reconciliation duration.
- Startup logs must separately report:
  - S3 `videos/` key listing duration and key count.
  - S3 video JSON load duration and object count.
  - Video status hydration duration and whether it was skipped for canonical bootstrap.
  - libSQL bulk insert duration and inserted/updated counts.
- Logs must be structured enough for Cloud Logging queries.
- Per-video cold bootstrap logs must be debug-level or replaced with a bounded aggregate summary.

### Snapshot Restore

- Startup must read `runtime-cache/libsql/current.json` before initializing local libSQL.
- The manifest must point to a compressed snapshot under `runtime-cache/libsql/snapshots/`.
- Startup must verify snapshot checksum and schema version before writing the local DB file.
- Startup must compare manifest source fingerprints against `videos/`, `user-preferences/`, and `tts-stats/` before restoring.
- If validation fails, startup must use the existing S3 rebuild path.
- After fallback rebuild or reconciliation changes, startup must publish a fresh snapshot manifest and compressed DB file.

### Production Mitigation Decision

- The plan should leave a clear operator choice for `min_instance_count = 1`.
- If implemented later, the warm-instance change must be explicit in Terraform/deploy config and documented with cost tradeoff.
- The active increment should not require the warm-instance change to prove value.

### Future Roadmap

The implementation should not block these later phases:

- A compact S3 catalog manifest for one-object cold bootstrap.
- Background reconciliation for non-critical status repair.
- A startup/readiness model that does not make Cloud Run's TCP probe appear ready while the app is still initializing.
- Possible split between serving and background worker responsibilities.

## Optimization Roadmap

### Phase 1: S3-Backed libSQL Snapshot Restore

This is the active increment.

Expected impact:

- Replaces thousands of S3 metadata/object reads with one manifest read, a few source-prefix list checks, and one compressed snapshot download when canonical data is unchanged.
- Preserves scale-to-zero by making the cold restore path cheaper.
- Keeps the current canonical S3 rebuild path as the recovery path.

### Phase 2: Keep One Warm Production Instance

Set backend `min_instance_count = 1` in production if the idle cost is acceptable. Google recommends minimum instances to reduce scale-from-zero latency for latency-sensitive services, with an explicit cost tradeoff: [Cloud Run minimum instances](https://cloud.google.com/run/docs/configuring/min-instances).

Expected impact:

- Prevents most user-visible cold starts.
- Does not fix deploy rollout or instance replacement startup time.

### Phase 3: Compact Catalog Manifest Or Delta Replay

Maintain a versioned `catalog/videos-manifest.json.gz` or equivalent object in S3. Cold startup should load this object first, then fall back to the current `videos/` scan if the manifest is missing or incompatible.

Expected impact:

- Collapses thousands of S3 `GET` requests into one bounded request.
- Keeps per-video snapshots for granular writes and recovery.

### Phase 4: Background Reconciliation

Serve the app after a minimal catalog is available and move expensive repair/reconciliation work into background tasks with explicit status. Cloud Run guidance recommends lazy initialization for infrequently used expensive work, while acknowledging first-request tradeoffs: [Cloud Run development tips](https://docs.cloud.google.com/run/docs/tips/general).

Expected impact:

- Improves first content even after instance replacement.
- Requires clear API behavior while the catalog is warming.

### Phase 5: Readiness Semantics

Revisit the current early TCP bind. Cloud Run startup probes can be TCP or HTTP; HTTP probes can express app-level readiness better than a raw open port: [Cloud Run health checks](https://cloud.google.com/run/docs/configuring/healthchecks).

Expected impact:

- Makes readiness, request logs, and cold-start diagnosis more truthful.
- Should happen after startup is fast enough or after warming states exist.

## Risks And Open Questions

- Trusting canonical video statuses during cold bootstrap may preserve stale statuses until a later repair job. This is acceptable only if normal write paths keep mirroring accurate statuses.
- libSQL transaction behavior needs confirmation with the local builder path used in Cloud Run.
- If status hydration is the largest cost, Phase 1 should be enough to cut startup sharply. If video JSON fan-out remains dominant, Phase 3 becomes the next priority.
- Keeping one warm Cloud Run instance is the fastest user-facing mitigation, but it adds idle cost and does not remove deploy-time cold starts.
- The `/api/mini` payload was about 190 KiB in the observed request. Payload slimming is likely lower priority than startup, but it may matter after cold start is fixed.

## Source Summary

- Cloud Run can reduce scale-from-zero latency with minimum instances, at added cost: [Cloud Run minimum instances](https://cloud.google.com/run/docs/configuring/min-instances).
- Startup CPU boost helps startup latency and is already enabled in this repo: [Cloud Run CPU configuration](https://docs.cloud.google.com/run/docs/configuring/services/cpu).
- Cloud Run logs include request, container, and system scaling logs that can support this measurement: [Cloud Run logging](https://cloud.google.com/run/docs/logging).
- S3 performance guidance favors measurement, concurrent requests, latest SDKs, retries, and minimizing latency-sensitive fan-out where possible: [S3 performance guidelines](https://docs.aws.amazon.com/AmazonS3/latest/userguide/optimizing-performance-guidelines.html).
- Firebase Hosting static content is already CDN-backed and this repo already sets immutable caching for `/_app/immutable/**`: [Firebase Hosting cache behavior](https://firebase.google.com/docs/hosting/manage-cache).
