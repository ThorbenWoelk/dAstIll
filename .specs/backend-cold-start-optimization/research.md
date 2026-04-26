# Backend Cold-Start Optimization Research

Research date: 2026-04-26

## Scope

This research covers production time-to-first-content for `https://dastill.web.app/mini`, with focus on:

- Cloud Run cold-start behavior for the backend.
- S3 and local libSQL startup hydration cost.
- Firebase Hosting static shell cost.
- Optimization options that fit the current one-instance backend architecture.

Out of scope:

- Horizontal backend scaling beyond one instance.
- Search worker redesign.
- Full migration away from S3-backed canonical storage.

## Live Evidence

The measured first load after the service had scaled to zero showed a real cold start:

- Firebase Hosting `/mini` HTML median TTFB/total from this machine: about 60 ms.
- Immutable app assets: about 276 KiB total; largest asset was 117.6 KiB CSS.
- Cloud Run request log for the cold preflight: `/api/mini` `OPTIONS` took 115.168 s.
- First successful `/api/mini` `GET` after startup took 1.365 s and returned about 190 KiB.
- Backend startup sequence:
  - `15:16:57.897` Cloud Run started a new instance.
  - `15:16:58.849` backend bound port 3001.
  - `15:16:58.867` TCP startup probe succeeded.
  - `15:16:59.025` local libSQL initialization started.
  - `15:18:52.908` SQL cache reconciliation completed with 3004 bootstrapped videos.
  - `15:18:53.371` Axum began serving.

Approximate split for the cold first backend path:

- Cloud Run platform startup until port bind: about 1.0 s.
- Application initialization after bind: about 114.5 s.
- First `/api/mini` handler execution after startup: about 1.4 s.
- Firebase static shell: about 0.06 s.

The dominant cost is not Firebase Hosting and not basic Cloud Run boot. It is synchronous backend startup hydration.

## Repo Findings

The backend Cloud Run service currently allows scale to zero and caps the backend at one instance:

- `terraform/cloud_run.tf` sets service `min_instance_count = 0`.
- Backend template scaling sets `max_instance_count = 1`.
- Startup CPU boost is already enabled.
- `docs/operations/deployment.md` explains the one-instance cap is intentional because the runtime keeps a local libSQL cache/index plus in-process background workers.

Startup does this before serving:

- Creates a local libSQL database under `/tmp`.
- Builds the AWS SDK clients using GCP-to-AWS Workload Identity Federation.
- Calls `reconcile_sql_cache_with_store()`.
- If local SQL has zero videos, loads every `videos/*.json` object from S3.
- Bulk-inserts videos into local libSQL.
- Starts workers and serves Axum only after reconciliation.

The video bootstrap path is expensive:

- `Store::load_all("videos/")` lists keys, then fetches each object with at most 12 concurrent S3 operations.
- `bootstrap_sql_videos_from_store()` turns all records into videos and calls `ts_bulk_insert_videos()`.
- `ts_bulk_insert_videos()` loops through videos and, for newly inserted rows, calls `hydrate_inserted_video_from_storage()`.
- `hydrate_inserted_video_from_storage()` does two extra S3 `HEAD` checks per video: one for `transcripts/{id}.json`, one for `summaries/{id}.json`.
- For 3004 bootstrapped videos, the cold path can perform roughly:
  - one `LIST` sequence for `videos/`
  - about 3004 `GET` requests for video snapshots
  - up to about 6008 `HEAD` requests for transcript and summary existence
  - 3004 individual libSQL insert executions
  - 3004 per-video info logs

This explains why the cold start is about two minutes even though S3 small-object latency is usually in the tens to hundreds of milliseconds.

## External Source Map

Primary sources reviewed:

- Google Cloud Run minimum instances: setting minimum instances keeps warm instances and reduces latency when scaling from zero, at added idle cost. Source: [Cloud Run minimum instances](https://cloud.google.com/run/docs/configuring/min-instances).
- Google Cloud Run startup CPU boost: provides extra CPU during startup and for 10 seconds after startup; this repo already enables it. Source: [Cloud Run CPU limits and startup CPU boost](https://docs.cloud.google.com/run/docs/configuring/services/cpu).
- Google Cloud Run health checks: startup probes determine readiness; TCP probes only prove the port accepts a connection, while HTTP probes can represent app-level readiness. Source: [Cloud Run health checks](https://cloud.google.com/run/docs/configuring/healthchecks).
- Google Cloud Run logging: request logs, container logs, and system scaling logs can be correlated; instance start reasons include `AUTOSCALING`, `DEPLOYMENT_ROLLOUT`, and min-instance starts. Source: [Cloud Run logging](https://cloud.google.com/run/docs/logging).
- Google Cloud Run development tips: startup time affects request latency when scaling from zero; tips include minimum instances, startup CPU boost, and lazy initialization of infrequently used objects. Source: [Cloud Run general development tips](https://docs.cloud.google.com/run/docs/tips/general).
- AWS S3 performance guidance: use concurrent requests, retries, latest SDKs, same-region compute where possible, and measure DNS/latency/transfer time. Source: [S3 performance guidelines](https://docs.aws.amazon.com/AmazonS3/latest/userguide/optimizing-performance-guidelines.html).
- AWS S3 performance design patterns: S3 scales by prefix and workload, but high request fan-out should use multiple connections and backoff on throttling. Source: [S3 performance design patterns](https://docs.aws.amazon.com/AmazonS3/latest/userguide/optimizing-performance-design-patterns.html).
- Firebase Hosting cache behavior: static content is cached on Firebase's global CDN; dynamic cache behavior is controlled through `Cache-Control`. Source: [Firebase Hosting cache behavior](https://firebase.google.com/docs/hosting/manage-cache).
- Firebase Hosting configuration: custom cache headers can be defined in `firebase.json`. Source: [Firebase Hosting full config](https://firebase.google.com/docs/hosting/full-config).

## Optimization Options

### 1. Keep One Warm Backend Instance

Set backend `min_instance_count = 1` in production.

Expected effect:

- Removes most user-visible scale-from-zero cold starts.
- Does not fix slow deploy rollout or instance replacement.
- Costs more because one idle instance remains warm.

Fit:

- Strong short-term mitigation.
- Compatible with the one-instance architecture.
- Should be guarded by environment/runtime mode so non-prod can still scale to zero.

### 2. Stop Synchronous Per-Video Hydration Before Serving

Make startup serve after minimal required initialization. Move expensive catalog reconciliation behind readiness or into background work.

Expected effect:

- Reduces cold startup from about 115 s toward a few seconds.
- Requires APIs to tolerate a cache warming state or to use a fast bootstrap snapshot.

Fit:

- Highest-impact product optimization.
- Must be designed carefully because `/api/mini`, workspace bootstrap, workers, and search may depend on local libSQL state.

### 3. Add a Compact Catalog Manifest

Write a single versioned, gzipped catalog manifest to S3 whenever video snapshots change. On cold start, load one object instead of listing and fetching thousands of `videos/*.json` files.

Expected effect:

- Collapses thousands of S3 calls to one or a small bounded set of calls.
- Keeps existing per-video snapshots as the durable granular store.
- Allows manifest versioning and fallback to the current scan path.

Fit:

- Strong match for current data model.
- Safer than downloading a raw SQLite/libSQL database file as the first increment.

### 4. Avoid Transcript/Summary HEAD Checks in Cold Bootstrap

When rebuilding from canonical video records, trust the stored `transcript_status` and `summary_status` fields or use manifest-provided status. Reconcile missing or stale statuses in a background repair job.

Expected effect:

- Removes up to two S3 `HEAD` calls per video from cold startup.
- Reduces both latency and S3 request cost.

Fit:

- High impact and local to the bootstrap path.
- Needs tests to ensure new video ingestion still performs status hydration when appropriate.

### 5. Batch libSQL Inserts in a Transaction

Wrap bulk insert/update operations in a transaction or use a libSQL batch path if available.

Expected effect:

- Reduces per-row write overhead.
- Makes startup less sensitive to catalog size.

Fit:

- Likely high value.
- Needs careful tests around existing-row preservation and status fields.

### 6. Reduce Startup Log Volume

Demote per-video `"inserted new video (libsql bulk)"` during cold bootstrap to debug or aggregate it.

Expected effect:

- Reduces Cloud Logging volume and startup overhead.
- Improves log readability.

Fit:

- Low risk after bootstrap metrics are added.

### 7. Fix Readiness Semantics

The current service binds the TCP port before initialization. Cloud Run's TCP startup probe succeeds quickly, then requests can sit queued on the socket while the app is not yet serving. Prefer one of:

- Bind only when the app is ready to serve.
- Serve a minimal router immediately with `/api/health` and `/api/startup-status`, but return explicit warming status for dependent APIs.
- Use an HTTP startup probe if app-level readiness should block traffic.

Expected effect:

- Makes logs and request latency easier to reason about.
- Does not by itself remove cold-start time.

Fit:

- Important observability and correctness improvement.
- Should be paired with faster startup or a warm instance to avoid long platform pending time.

### 8. Keep Firebase Static Tuning As Secondary

Firebase Hosting is already configured with immutable caching for `/_app/immutable/**` and `no-cache` for `index.html`. The measured static shell is not the bottleneck.

Expected effect:

- Minor. Revisit only if bundle or CSS size becomes a separate concern.

Fit:

- No immediate action required for this cold-start problem.

## Recommended Sequence

1. Add observability around startup phases and catalog bootstrap counts.
2. Change production backend to keep one warm instance if the cost is acceptable.
3. Remove per-video transcript/summary `HEAD` checks from cold bootstrap.
4. Batch libSQL bulk inserts and reduce per-video info logs.
5. Add a compact catalog manifest and make cold startup prefer it.
6. Move long reconciliation/search hydration to background paths with explicit warming status.
7. Revisit startup probe/readiness semantics after the cold path no longer takes minutes.

## Gaps

- Current logs do not split S3 `LIST`, S3 `GET`, S3 `HEAD`, JSON decode, and libSQL insert timing.
- The local production proxy token was not valid for direct API timing, so successful `/api/mini` handler timing was taken from Cloud Run request logs rather than synthetic authenticated curl.
- Exact cost for one warm instance should be estimated against current Cloud Run pricing and expected monthly idle hours before changing Terraform.
