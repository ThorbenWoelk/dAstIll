# Tasks: Vocabulary Replacements

## Current State
Backend and frontend implementation are in place, and the staged pre-commit checks passed.

## Steps
- [x] Inspect current selection, preferences, and summary generation flows.
- [x] Write failing tests for vocabulary replacement persistence and summary application.
- [x] Implement backend preference model and summary replacement pipeline.
- [x] Implement frontend correction action and preference persistence updates.
- [x] Run format, lint, test, build, and pre-commit verification.

## Decisions Made During Implementation
- Initial scope stores global user-level replacements in `UserPreferences`.
- Summary generation will apply literal exact-string replacements centrally before prompting the summarizer.
- The inline correction action reuses the existing selection tooltip and captures the canonical replacement with a simple prompt instead of introducing a new management surface.
