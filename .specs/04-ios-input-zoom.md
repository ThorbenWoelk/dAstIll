# Spec 04: iOS Input Auto-Zoom Fix

Priority: **P0**
Scope: All text inputs and textareas

## Problem

iOS Safari automatically zooms the page when a user focuses an input with `font-size < 16px`. Six inputs in the app use font sizes from 12-15px, causing an involuntary zoom on every focus event. Users must pinch-to-zoom back out, which is disorienting and breaks layout flow.

## Affected Inputs

| File | Line | Current Size | Element |
|------|------|-------------|---------|
| `AddSourceDrawer.svelte` | 361 | `text-sm` (14px) | URL/source input |
| `VocabularyReplacementModal.svelte` | 121 | `text-[15px]` | Replacement text input |
| `ChatInput.svelte` | 716 | `text-[14px]` | Chat message textarea |
| `QueueContentPanel.svelte` | 315 | `text-[12px]` | Schedule date input |
| `WorkspaceSidebarSyncDateControl.svelte` | via inputClass | `text-[12px]` | Sync date input |
| `ChannelOverviewMainContent.svelte` | 194 | `text-sm` (14px) | Date input |

## Fix

Set all input and textarea font sizes to at least 16px on mobile. Two approaches:

### Option A: Global CSS rule (recommended)
Add to `app.css` inside the `@media (max-width: 1023px)` block:

```css
input, textarea, select {
  font-size: 16px;
}
```

This is a blunt but effective fix. Desktop sizes remain unchanged since the media query scopes it to mobile.

### Option B: Per-input Tailwind classes
Replace each instance:
- `text-sm` -> `text-base max-lg:text-base` (or just `text-base`)
- `text-[14px]` -> `max-lg:text-base text-[14px]` (16px on mobile, 14px on desktop)
- `text-[15px]` -> `text-base` (just use 16px everywhere)
- `text-[12px]` -> `max-lg:text-base text-[12px]`

### Recommendation

Use Option A for its simplicity and completeness. It catches any future inputs automatically. The visual difference between 14px and 16px is minimal and not worth the zoom tradeoff.

## Additional Input Improvements

While fixing font sizes, also address:

### Radio and checkbox touch targets
- `AddSourceDrawer.svelte` lines 453-466: `<input type="radio" class="mr-2">` — default browser radio buttons are ~13px. Wrap each radio/checkbox + label in a `<label>` element with `flex items-center gap-3 py-3 min-h-[44px]` so the entire row is tappable.
- `AddSourceDrawer.svelte` lines 505-509: Same for checkboxes.

### Date inputs
- Native date pickers on iOS render as wheel spinners. The 12px font size on `QueueContentPanel.svelte` and `WorkspaceSidebarSyncDateControl.svelte` makes the trigger text nearly invisible. Bumping to 16px also fixes readability of these native controls.

## Verification

- On iOS Safari (real device or BrowserStack): focus each affected input and confirm no page zoom occurs
- On Android Chrome: confirm inputs remain usable (Android doesn't auto-zoom but benefits from larger text)
- Visual review: confirm 16px inputs look proportional in their containers
- `bun run check`, `bun run test` pass
