# Tasks: Databricks Analytics Showcase

## Current State
Frontend analytics instrumentation already exists in the repo. The backend ingestion path now enqueues analytics batches and writes them to Databricks in a background worker so user requests do not wait on the warehouse, including during serverless cold starts.

## Steps
- [x] Create frontend analytics module (`src/lib/analytics/events.ts`, `tracker.ts`, `read-time.ts`)
- [x] Wire tracker into `src/routes/+page.svelte` for the starter event set
- [x] Add `POST /api/analytics/events` ingestion endpoint in Rust
- [x] Register analytics handler in `handlers/mod.rs` and `main.rs`
- [x] Implement mobile-safe summary read-time tracking with touch and visibility handling
- [x] Replace the S3 analytics sink with a Databricks SQL Statement Execution sink
- [x] Add Databricks runtime config, lazy schema/table creation, and backend deployment env wiring
- [x] Verify: `cargo check` clean
- [x] Verify: `bun run check` clean
- [ ] Decide where Silver/Gold SQL models will live in-repo for the direct-to-Delta workflow
- [ ] Create the initial Databricks dashboards for the three selected use cases
- [ ] Validate the read-time metric on desktop and mobile behavior before presenting findings

## Decisions Made During Implementation

- **Free Edition ingestion path**: Write analytics events directly into Databricks Delta via the SQL Statement Execution API instead of S3 + Auto Loader.
- **Bronze table creation**: The backend lazily creates the target schema and Bronze table with `CREATE SCHEMA IF NOT EXISTS` and `CREATE TABLE IF NOT EXISTS`.
- **Non-blocking ingestion**: The analytics endpoint only validates and enqueues; it returns immediately and never waits for Databricks completion on the request path.
- **Cold-start handling**: Databricks writes run in a background worker with retry/backoff and long polling so serverless warehouse startup does not block or fail the caller immediately.
- **summary_id**: Uses `video_id` as `summary_id` because the backend does not currently expose a separate summary entity id.
- **Event idempotency**: The frontend now generates `event_id`; the backend falls back to a deterministic hash of the raw event payload if it is missing.
- **Gold ranking metric**: Use `active_time_ms` or an engagement-weighted score based on scroll depth instead of raw open time.
- **Heartbeat fallback**: `summary_heartbeat` remains optional and should only be enabled if close-event delivery proves unreliable.
- **Auth**: Analytics ingestion remains inside `protected_api` and uses the same proxy-auth and rate-limit path as the rest of the backend.
