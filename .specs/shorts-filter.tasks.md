# Tasks: Shorts Filtering

## Current State
Implemented backend Shorts metadata + filtering and wired frontend hide-shorts toggles. Verified with backend tests and API smoke checks.

## Steps
- [x] Add failing backend tests for Shorts classification/filtering (Red).
- [x] Implement backend `is_short` model, DB migration, classification, and API filtering (Green).
- [x] Add frontend toggle wiring to hide Shorts on video lists.
- [x] Run format/check/tests for backend and frontend.
- [ ] Manually verify hide-Shorts behavior in app.

## Decisions Made During Implementation
- Shorts classification is derived from YouTube `/shorts/{id}` redirect/canonical behavior.
- API keeps default behavior (`include_shorts=true`) and enables filtering only when explicitly requested.
