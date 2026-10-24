# Local YouTube fetching and hosted generation

## Stories

1. Hosted processing requests missing YouTube transcripts through GCS, without
   contacting YouTube caption endpoints or consuming failure retries while the Mac is offline.
2. The local app fetches requested transcripts without an active browser. Requests and
   results survive restarts; temporary errors back off. The hosted process imports results,
   then summarizes and evaluates using its current prompts and model configuration.
3. Local fetching does not publish a stale SQL snapshot or overwrite hosted video metadata.
   Podcasts and other sources keep their hosted processing paths.
4. Clearly corrupted stored articles can be regenerated from existing transcripts while
   retaining the previous content until generation succeeds.

## Tests

- Role parsing rejects invalid values; remote never calls the YouTube extractor.
- Two stores with separate SQL caches share transcript requests/results and import a result.
- Waiting jobs do not consume retries or block ready summary work.
- Retry deadlines survive a worker restart; completed jobs are not fetched again.
- Reset/deletion cancels a handoff; stale results cannot replace a new request.
- Hosted generation/evaluation still run; local fetching ignores browser activity.
- All backend/frontend checks, audits, launcher compilation and installation before shipping.

## Storage

Keep the existing regional Standard GCS bucket. It already holds canonical video metadata,
transcripts, summaries, and recoverable SQL snapshots. Add only small request/result objects;
no paid database service or storage migration is needed.

## Verification results

- 661 backend tests passed; 8 existing opt-in tests were not run.
- Frontend formatting, lint, type checking, 85 unit tests, 29 E2E tests, build, and production audit passed.
- Backend audit passed with two existing allowed warnings; docs build and macOS launcher tests/compilation passed.
- Integration test uses separate SQL databases with shared objects, verifies waiting does not
  consume retries or starve a completed job, then runs summary generation and evaluation.
- Live GCS inspection confirmed regional Standard storage with versioning. The 3,594-summary
  audit selected 43 clearly corrupted articles, excluding legitimate quoted reasoning.
