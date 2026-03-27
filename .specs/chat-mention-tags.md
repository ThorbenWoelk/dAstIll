# Chat Mention Tags

## Summary

Render scoped chat mentions as minimal tags in both the composer and user message bubbles.

Supported forms:
- `@{Channel Name}` -> `Channel Name`
- `+{Video Title}` -> `Channel Name - Video Title`

## Goals

- Keep the stored message text unchanged so backend parsing and scope resolution continue to work.
- Replace raw mention syntax with a cleaner visual treatment in the chat UI.
- Use one shared parsing and resolution path for both the composer and sent user messages.
- Reuse existing chat suggestion endpoints instead of adding new backend APIs.

## Approach

1. Parse braced mentions from chat text in a frontend utility.
2. Resolve mention labels through existing suggestion endpoints:
   - channels via `/api/chat/suggestions/channels`
   - videos via `/api/chat/suggestions/videos`
3. Cache resolutions client-side by mention token.
4. Render minimal pills:
   - channel tags show channel name
   - video tags show `channel - title`
5. Show tags in the composer for mentions present in the draft.
6. Render tags inline inside user message bubbles.

## Non-Goals

- Changing backend message storage format
- Replacing the textarea with a rich text editor
- Rendering tags inside assistant markdown responses

## Acceptance

- Typing or inserting `@{HealthyGamerGG}` shows a minimal channel tag in the chat composer.
- Typing or inserting `+{3 Things Every Relationship Has}` shows a minimal video tag in the chat composer as `HealthyGamerGG - 3 Things Every Relationship Has`.
- After send, the user message bubble renders the same tags instead of raw mention syntax.
- Unknown mentions degrade safely to their raw text.
