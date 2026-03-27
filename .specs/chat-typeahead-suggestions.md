# Chat Typeahead Suggestions

## Goal

Add inline type suggestions to the chat input:

- `@` suggests channels
- `+` suggests videos

Suggestions should filter as the user types, support keyboard navigation, and insert a canonical scoped token back into the input.

## UX

- Typing `@` opens channel suggestions.
- Typing `@hea` filters channels like `HealthyGamerGG`.
- Typing `+` opens video suggestions.
- Typing `+dop` filters videos by title.
- `ArrowUp` and `ArrowDown` move through suggestions.
- `Enter` and `Tab` accept the selected suggestion.
- `Escape` closes suggestions.
- Clicking a suggestion inserts it.
- Inserted syntax should be stable and parseable:
  - channels: `@{HealthyGamerGG}`
  - videos: `+{Why Effort Alone Doesn’t Lead to Change}`

## Backend

- Add read-only chat suggestion endpoints:
  - `GET /api/chat/suggestions/channels?q=<query>&limit=<n>`
  - `GET /api/chat/suggestions/videos?q=<query>&limit=<n>`
- Ranking:
  - exact/prefix match before substring match
  - videos tie-break by recency
- Extend mention parsing so `+` is video-only scope while `@` stays channel-first.

## Frontend

- Detect active trigger token from the textarea caret position.
- Debounce suggestion fetches.
- Render a suggestion popover inside the chat input container.
- Replace the active token with the selected canonical mention plus trailing space.

## Verification

- Backend unit tests for ranking and mention parsing.
- Frontend typecheck.
- Manual verification for:
  - `@hea`
  - `+eff`
  - arrow key navigation
  - enter/tab accept
  - escape close
  - message send still works after insertion
