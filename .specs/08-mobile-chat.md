# Spec 08: Mobile Chat Optimization

Priority: **P1**
Scope: Chat page and related components

## Problems

1. **Input controls stack vertically** below 640px (`ChatInputControls.svelte:34`): the source selector, deep research toggle, and submit button all stack into a tall column, eating viewport space. On a short phone screen (667px viewport height minus keyboard ~300px minus top bar 48px minus input area), very little room remains for messages.

2. **Chat sidebar hidden on mobile** (`chat/+page.svelte`): `hidden lg:flex lg:h-full` — entirely invisible. Mobile conversation access is through `ChatMobileConversationsOverlay`, which works but:
   - The overlay trigger needs to be discoverable (currently in `ChatContentSectionHeader`)
   - Conversation rename/delete buttons are hover-only (spec 03)

3. **Message area padding** is heavy on mobile: `px-4 sm:px-6` plus the 52px nav rail = significant horizontal padding eating into message width.

4. **Jump-to-latest button** at `h-9` (36px) — below touch target minimum.

## Requirements

### Compact mobile input layout
- On mobile, render the chat input controls in a single row:
  - Source selector as a small icon/chip (not full dropdown)
  - Deep research toggle as a small icon toggle
  - Submit button at the right
- The textarea should take full width above the controls row
- Total input area height when not focused: ~80px max (textarea + controls row)

### Conversation access
- Add a visible "Conversations" button in the mobile top bar (left side, where the back button is) that opens `ChatMobileConversationsOverlay`
- Make sure the overlay shows conversation management actions (rename, delete) — see spec 03

### Message area
- With nav rail hidden (spec 07), reclaim the 52px for message content
- Reduce horizontal padding on mobile: `px-3` instead of `px-4 sm:px-6`
- Ensure messages use the full available width

### Jump-to-latest
- Increase button size to 44px minimum touch target

## Files to Modify

| File | Change |
|------|--------|
| `ChatInputControls.svelte` | Compact single-row layout on mobile |
| `chat/+page.svelte` | Reduce message padding; add conversation trigger to top bar |
| `ChatMobileConversationsOverlay.svelte` | Ensure management actions are accessible |
| `MobileYouTubeTopNav.svelte` | Add conversation list trigger slot |

## Verification

- Chat input stays compact on mobile, leaving maximum viewport for messages
- Conversations are accessible via a visible button
- Messages use full width without excessive padding
- `bun run check`, `bun run test` pass
