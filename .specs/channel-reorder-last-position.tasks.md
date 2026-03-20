# Tasks: Channel reorder last-position drop

## Current State

The fix is implemented and validated at the unit, typecheck, and build layers. The sidebar now exposes a real drop area below the final channel row, and the reorder helper supports explicit moves into the last list position. Headed browser drag verification was attempted but not completed without adding extra Playwright dependencies to the repo.

## Steps

- [x] Inspect the sidebar drag/drop code and isolate the likely root cause.
- [x] Write a dedicated spec and tasks file under `.specs/`.
- [x] Add a failing regression test for dropping a channel into the last position.
- [x] Update the reorder/drop logic to support appending at the end of the list.
- [x] Run targeted frontend validation for the new reorder path.

## Decisions Made During Implementation

- Keep the fix within the frontend reorder helpers and sidebar drop flow unless validation proves a lower-level state issue.
- Model the end-of-list drop as an explicit move-to-index operation so the UI can target the final slot without inventing a fake channel id.
- Treat the user-reported failure as a drop-zone gap, not just a reorder math problem: the final slot needs its own drop target in the list container.
- Keep browser automation out of the repo for this fix. The attempted headed verification path was discarded rather than adding new test dependencies for a one-off check.
