# Vocabulary Replacements

**Linear:** none

## Problem

Summaries currently rely only on the raw transcript text. If a transcript misspells a person, place, company, or technical term, later summary runs can repeat that bad spelling because there is no user-controlled correction memory.

## Goal

Users can mark a selected phrase as incorrect, save a canonical replacement, and have future summary generation account for those saved replacements automatically.

## Requirements

- The app must persist a vocabulary list of user-defined replacements in the existing preferences surface so the list survives reloads and later sessions.
- While reading transcript or summary content, the user must be able to select text and trigger a correction flow distinct from saving a highlight.
- The correction flow must let the user store a `replace x -> y` rule based on the selected phrase and the user-entered canonical spelling.
- Future summary generation and regeneration must apply the saved vocabulary replacements before prompting the summarizer so canonical spellings are reflected in generated summaries.
- Existing highlight behavior and existing preferences behavior must continue to work.
- The new backend and frontend behavior must be covered by automated tests.

## Non-Goals

- Full transcript editing from the correction flow.
- Per-channel or per-video vocabularies.
- Fuzzy replacement matching, phonetic matching, or model-driven entity resolution.
- A dedicated vocabulary management screen in this first scope.

## Design Considerations

- `UserPreferences` already provides a singleton persistence surface, which keeps this feature small and avoids introducing a new collection.
- Summary generation should apply replacements in one central backend path so queue processing and manual regeneration behave consistently.
- The transcript selection UI already exposes a floating action for highlights, so adding a second action there is the least disruptive interaction model.

## Open Questions

- Whether future iterations should expose a dedicated UI to review and delete saved vocabulary rules outside the inline correction flow.
