# Tasks: Gap Scan Storage Reconcile

## Current State
Gap-scan now reconciles newly inserted Firestore video rows against existing transcript/summary storage artifacts, and startup queue healing also upgrades already-inserted stale rows. Focused backend tests and `cargo check` are passing.

## Steps
- [x] Confirm the gap-scan insert path and the default status assignment for new video rows
- [x] Patch new video insertion to reconcile statuses from stored transcript/summary artifacts
- [x] Add regression coverage for inserted videos with existing stored artifacts
- [x] Verify targeted backend tests pass
