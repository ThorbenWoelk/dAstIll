# Tasks: Graceful handling for transcript formatting timeouts

## Current State

Production behavior was reproduced and the mitigation is implemented. Backend transcript formatting now leaves headroom before the 300 second upstream cutoff, the frontend keeps a grace window beyond that limit, and targeted plus broad local validation passed.

## Steps

- [x] Inspect the production formatting path and locate the timeout boundaries.
- [x] Reproduce the issue against the live backend with a long transcript payload.
- [x] Reduce backend formatting timeout to leave response headroom before the upstream cutoff.
- [x] Add frontend grace so backend timeout responses are not masked by the browser abort.
- [x] Update timeout copy and add targeted regression tests.
- [x] Run format, typecheck, and targeted tests.

## Decisions Made During Implementation

- Prefer graceful degradation over increasing infra timeouts. The formatting path is already expensive for long transcripts, so bounded failure is safer than allowing much longer synchronous waits.
- Keep the change localized to timeout coordination and user-facing messaging. Do not redesign transcript formatting in this fix.
