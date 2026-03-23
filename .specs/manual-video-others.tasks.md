# Tasks: Manual Video Ingest Into Others

## Current State

Backend and frontend support for manual video ingest is implemented and validated with Rust tests, Svelte check, and targeted Bun tests.

## Steps

- [x] Inspect current channel list, snapshot, and sidebar flows
- [x] Add failing tests for `Others` grouping and manual-video target resolution
- [x] Implement backend synthetic `Others` channel and manual video ingest endpoint
- [x] Update frontend add-source input and selection flow
- [x] Run targeted validation for backend and frontend changes

## Decisions Made During Implementation

- Reuse the existing sidebar add input instead of introducing a second entry point.
- Model `Others` as a synthetic channel entry so existing selection and snapshot APIs can keep their shape.
- Keep `Others` virtual by keying on `__others__` instead of adding a new persisted channel type.
