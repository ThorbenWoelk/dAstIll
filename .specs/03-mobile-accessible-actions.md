# Spec 03: Mobile-Accessible Actions

Priority: **P0**
Scope: Channel deletion, chat conversation management, hover-reveal controls

## Problem

Several destructive and management actions are completely inaccessible on mobile:

1. **Channel deletion** — `ChannelCard.svelte` line 124: delete button has `max-lg:hidden`, entirely removed from the DOM on mobile. No alternative flow exists.
2. **Chat conversation rename** — `ChatSidebar.svelte` line 159: rename button is `opacity-0 group-hover:opacity-100` with no `focus-within` or touch fallback.
3. **Chat conversation delete** — `ChatSidebar.svelte` line 180: same hover-reveal pattern, no touch fallback.
4. **Chat message copy** — `ChatMessage.svelte` line 134: copy button is `group-hover/copy:opacity-100`, invisible to touch users unless they have `prefers-reduced-motion` set.

## Requirements

### Channel Deletion
- Add a mobile-accessible delete flow. Two options:
  - **Option A (recommended)**: Long-press on channel card reveals a context action sheet (bottom sheet) with "Delete Channel" option
  - **Option B**: Add a visible delete button in the channel card's mobile layout, positioned to avoid accidental taps (e.g., trailing icon with confirmation modal)
- Remove `max-lg:hidden` from the delete button or replace with the chosen pattern
- Must still show `ConfirmationModal` before actual deletion

### Chat Conversation Management
- Make rename and delete buttons always visible in the chat sidebar conversation list on mobile
- Replace `opacity-0 group-hover:opacity-100` with a pattern that works on touch:
  - Always show action buttons at reduced opacity (e.g., `opacity-50`) on mobile
  - Or use swipe-to-reveal pattern on conversation rows
  - Or add a "..." overflow menu button that opens an action sheet
- The `ChatMobileConversationsOverlay` already exists for mobile — ensure these actions work within it

### Chat Message Copy
- Make the copy button always visible on mobile (not hover-gated)
- Use `max-lg:opacity-100` to override the hover-reveal on touch screens
- Or add a long-press-to-copy interaction on the message text

## Files to Modify

| File | Change |
|------|--------|
| `ChannelCard.svelte` | Remove `max-lg:hidden` from delete button; add mobile action pattern |
| `ChatSidebar.svelte` | Make rename/delete buttons touch-accessible |
| `ChatMobileConversationsOverlay.svelte` | Ensure conversation management actions are available |
| `ChatMessage.svelte` | Make copy button visible on touch devices |

## Implementation Notes

- For the long-press pattern, use a `touchstart`/`touchend` timer (500ms threshold). Cancel on `touchmove` to distinguish from scroll.
- For the bottom sheet / action sheet pattern, create a reusable `MobileActionSheet.svelte` component if one doesn't exist. It should:
  - Render as `fixed bottom-0 left-0 right-0 z-[80] lg:hidden`
  - Show action items as full-width buttons (44px+ height each)
  - Include a cancel/dismiss button
  - Use opaque `var(--surface)` background
  - Animate up from bottom edge (200ms ease-out)
- For swipe-to-reveal, use the existing `swipe.ts` infrastructure pattern but applied horizontally on list rows.

## Verification

- On mobile (< 1024px), can delete a channel via the channel card
- On mobile, can rename and delete a chat conversation
- On mobile, can copy a chat message
- All actions show confirmation where appropriate
- No hover-only gates remain on destructive actions
- `bun run check`, `bun run test` pass
