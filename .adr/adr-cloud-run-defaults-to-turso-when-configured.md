# ADR: Cloud Run Must Default To Turso When Turso Credentials Exist

## Status

Accepted

## Context

Production runs the backend on multiple Cloud Run instances.
The backend stores canonical video rows in libSQL and uses those rows for direct
video lookup, transcript lookup, summary lookup, and highlight creation.

When Cloud Run starts without Turso enabled, each instance falls back to its own
local libSQL file. That creates instance-local `videos` state. A backfill or sync
request can populate one instance while another instance still returns `404 Video not found`
for the same video id.

That split caused intermittent production highlight failures where the same
video id alternated between `201` and `404` depending on which instance handled
the request.

## Decision

Cloud Run should treat Turso as the production default when Turso credentials are present.

Concretely:

- the release workflow sets `START_APP_USE_TURSO=1`
- the backend config also defaults Turso on in Cloud Run when `TURSO_DB_URL` and
  `TURSO_AUTH_TOKEN` are present, even if `START_APP_USE_TURSO` was omitted

Local development keeps explicit opt-in behavior so engineers can still choose
between local libSQL and Turso-backed runs intentionally.

## Consequences

Positive:

- all Cloud Run instances read from the same Turso-backed source of truth
- canonical video lookup stops depending on which instance handled the request
- deployment misconfiguration becomes much harder to miss

Tradeoffs:

- Cloud Run now depends on Turso availability whenever Turso credentials are configured
- local behavior and Cloud Run behavior differ slightly when `START_APP_USE_TURSO` is omitted

## Directive

Do not rely on per-instance local libSQL for canonical production video state.
If Cloud Run is configured with Turso credentials, keep Turso enabled by default
unless production architecture is intentionally redesigned first.
