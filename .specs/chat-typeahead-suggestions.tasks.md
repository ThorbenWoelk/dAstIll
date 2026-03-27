# Tasks: Chat Typeahead Suggestions

## Current State
The feature is implemented. Backend chat suggestion endpoints, `+` video mention parsing, and the inline Svelte typeahead UI are all in place and verified with tests and builds.

## Steps
- [x] Create spec and task files.
- [x] Add backend suggestion response/query models and endpoints.
- [x] Add backend ranking/filtering tests for channel and video suggestions.
- [x] Extend mention parsing so `+` scopes videos.
- [x] Add frontend API helpers and types for chat suggestions.
- [x] Implement inline chat input typeahead UI with keyboard navigation and insertion.
- [x] Run verification: backend tests, frontend check, release build, pre-commit.

## Decisions Made During Implementation
- Insert canonical mention syntax with braces to avoid quoting ambiguity.
- `@` is channel-first.
- `+` is video-only.
- Suggestions appear for bare trigger tokens like `@hea` and `+eff`, then insert canonical braced mentions on acceptance.
