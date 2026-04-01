# Start App Fast Fail

## Status
Accepted

## Context

`./start_app.sh` waits for HTTP health checks after launching backend, frontend, and docs. When one of those processes exits immediately during boot, the script continues polling until timeout. In practice this makes obvious startup failures look like the script is hung after a `Starting ...` line.

Local backend startup also depends on valid Firestore credentials. When those credentials are missing or stale, the backend exits before the health endpoint is reachable, so the startup script should surface that failure quickly and the local-development docs should explain the requirement clearly.

## Decision

- Update `start_app.sh` so readiness checks fail fast if the child process exits before the target URL becomes healthy.
- Keep the existing log-tail behavior, but trigger it immediately on early process exit instead of waiting for the retry budget to drain.
- Document the Firestore local-auth prerequisite and the expected remediation paths for a missing or stale `GOOGLE_APPLICATION_CREDENTIALS` setting.

## Consequences

- Startup failures become visible immediately instead of looking like a hang.
- Developers get faster feedback from `./start_app.sh` and `./start_app.sh --detach`.
- The app still will not start without valid backend Firestore auth, but the failure becomes actionable instead of opaque.
