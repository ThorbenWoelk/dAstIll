# Tasks: Graceful handling for transcript formatting timeouts

## Current State

Production behavior is reproduced. A long transcript formatting request hits `HTTP 504` at about 300.08 seconds, while the browser abort path is configured to fire at 300 seconds exactly.

## Steps

- [x] Inspect the production formatting path and locate the timeout boundaries.
- [x] Reproduce the issue against the live backend with a long transcript payload.
- [ ] Reduce backend formatting timeout to leave response headroom before the upstream cutoff.
- [ ] Add frontend grace so backend timeout responses are not masked by the browser abort.
- [ ] Update timeout copy and add targeted regression tests.
- [ ] Run format, typecheck, and targeted tests.

## Decisions Made During Implementation

- Prefer graceful degradation over increasing infra timeouts. The formatting path is already expensive for long transcripts, so bounded failure is safer than allowing much longer synchronous waits.
- Keep the change localized to timeout coordination and user-facing messaging. Do not redesign transcript formatting in this fix.
