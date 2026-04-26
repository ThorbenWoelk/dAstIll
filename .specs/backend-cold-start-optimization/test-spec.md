# Backend Cold-Start Optimization Test Spec

## Acceptance Criteria

- Startup restores a valid compressed libSQL snapshot from `runtime-cache/libsql/` before opening local libSQL.
- Startup rejects missing, corrupt, stale, or schema-incompatible snapshots and falls back to the canonical S3 rebuild path.
- Snapshot manifests include source fingerprints for `videos/`, `user-preferences/`, and `tts-stats/`.
- Startup publishes a fresh derived snapshot after fallback rebuild or reconciliation changes.
- Canonical S3 objects remain the source of truth.
- Existing `/api/mini` and `/api/workspace/bootstrap` behavior remains unchanged after startup.

## Proof For The Current Increment

1. Add unit tests around snapshot compression, checksum, and source-fingerprint comparison.
2. Run backend checks.
3. Deploy or run a production-like cold-start validation and compare Cloud Run logs before and after.

Target proof threshold:

- First cold start after deployment may rebuild from canonical S3 data and publish a snapshot.
- Second cold start with unchanged canonical data restores from snapshot.
- Cold startup total duration improves materially from the observed 115 s baseline on the snapshot restore path, or logs identify the next dominant cost.

## Automated Checks

Backend:

- `cd backend && cargo check`
- `cd backend && cargo test`

Focused tests to add or update:

- Snapshot gzip round trip preserves DB bytes.
- Snapshot checksum is stable.
- Prefix source state changes when key count or latest modified timestamp changes.
- A stale source fingerprint prevents restore and uses the fallback rebuild path.

Optional frontend checks only if `/mini` or API client behavior changes:

- `cd frontend && bun run check`
- `cd frontend && bun run test`

## Manual Checks

Run a cold-start validation in production or a staging-equivalent project:

1. Deploy the backend.
2. Ensure no warm backend instance exists, unless testing the explicit `min_instance_count = 1` mitigation.
3. Open `https://dastill.web.app/mini` or call the first authenticated `/api/mini` path.
4. Query Cloud Run logs for:
   - system instance start reason
   - startup phase timings
   - first `/api/mini` latency
   - first `/api/workspace/bootstrap` latency, if exercised
5. Compare against the observed baseline:
   - new instance start: `2026-04-26T15:16:57.897Z`
   - serving began: `2026-04-26T15:18:53.371Z`
   - cold preflight latency: `115.168 s`
   - first `/api/mini` GET latency: `1.365 s`
   - bootstrapped videos: `3004`

Useful Cloud Logging query shape:

```bash
gcloud logging read \
  'resource.type="cloud_run_revision" AND resource.labels.service_name="dastill-backend" AND timestamp >= "<start>"' \
  --project=dastill \
  --format='table(timestamp,severity,httpRequest.requestUrl,httpRequest.status,httpRequest.latency,jsonPayload.message,jsonPayload.duration_ms,jsonPayload.count)'
```

## Edge Cases

- Empty `videos/` prefix.
- Missing compact or canonical video data.
- Video record with `summary_status=ready` but missing summary object.
- Video record with stale status after a failed write.
- Existing local libSQL state is non-empty, so cold bootstrap should not rerun.
- Partial S3 failure during video object load.
- Missing, corrupt, or schema-incompatible runtime-cache snapshot.
- Snapshot exists but canonical source prefixes changed after publication.
- Service starts while a user request is already pending.

## Observability Or Failure Signals

- Cloud Run system log reason `AUTOSCALING` followed by long request latency.
- Snapshot restore logs `libSQL snapshot restored` on the fast path.
- Snapshot restore logs fallback reasons when the manifest is missing, stale, or corrupt.
- `SQL cache reconciliation complete` still reports more than 30 s after a valid restore.
- Any increase in `/api/mini` 5xx, 401/403 aside from expected auth, or response size.
- Any mismatch between video statuses in local libSQL and S3 canonical records after background reconciliation.

## Stop Line

Stop after Phase 1 when:

- Snapshot restore and publish code is present.
- The existing S3 rebuild path remains the fallback.
- Snapshot validation covers checksum, schema, and source freshness.
- Existing API behavior is preserved.
- A new cold-start measurement identifies the next bottleneck.

Do not proceed to background reconciliation, worker splitting, or Terraform warm-instance changes in the same implementation unless explicitly approved.
