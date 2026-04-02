# Start App Runtime Readiness

## Status
Accepted

## Context

`./start_app.sh` can report success once the dev servers answer on their ports, even when the
workspace API is returning startup-time `500` responses. A recent Firestore video-read change
also made malformed legacy `dastill_videos` rows fatal for collection reads, which blocks the
workspace bootstrap endpoint and leaves the app unusable during local startup.

## Decision

- Make Firestore video collection reads tolerant of malformed legacy rows by skipping them with a
  warning instead of failing the entire query.
- Treat malformed single-document video reads as absent so request handlers degrade to missing data
  instead of surfacing `500` responses for the whole workspace.
- Update `start_app.sh` to verify the backend workspace bootstrap endpoint before it reports that
  the app is ready.

## Consequences

- One malformed canonical video record no longer takes down local startup or workspace reads.
- `./start_app.sh` only reports success when the backend can serve the workspace shell's initial
  data path, not just the health endpoint.
- Malformed legacy rows are still visible in logs and should still be cleaned up, but they stop
  being a full-app outage.
