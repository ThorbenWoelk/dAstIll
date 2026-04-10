# Spec 05: Hover-to-Touch Migration

Priority: **P1**
Scope: All hover-only UI patterns

## Problem

Several UI elements are only accessible via mouse hover and have no touch alternative. Touch users cannot discover or interact with these controls.

## Affected Patterns

### 1. Tooltip-only labels on collapsed nav (`WorkspaceNavRail.svelte:187`)

**Current**: When nav is collapsed (52px, always on mobile), section labels are shown only via `data-tooltip` on hover.
**Impact**: Touch users see 5-6 unlabeled icons with no way to identify them.
**Fix**: Addressed in [spec 02](./02-mobile-bottom-nav.md) (bottom nav replaces the rail on mobile) and [spec 07](./07-mobile-nav-rail.md).

### 2. Collapsed sidebar channel tooltips (`WorkspaceSidebarCollapsedChannelRail.svelte:48`)

**Current**: Channel names shown only via `data-tooltip` on hover in collapsed rail.
**Impact**: Touch users see channel avatars with no name.
**Fix**: Add visible channel name below/beside avatar, or ensure the collapsed rail is not the default mobile state (the full overlay browse pattern already handles this).

### 3. Video row hover preview (`WorkspaceSidebarVideoRow.svelte`)

**Current**: `onmouseenter`/`onmouseleave` triggers a preview state on hover.
**Impact**: Touch users have no equivalent preview mechanism.
**Fix**: Make the tap action itself navigate to the video (which it already does). Remove hover preview dependency. If preview is valuable, trigger it on long-press or show a small preview icon that expands on tap.

### 4. Content action button tooltips (`ContentActionButton.svelte:52-55`)

**Current**: `data-tooltip` describes button function, hover-only.
**Impact**: Touch users cannot discover what action buttons do.
**Fix**: Already addressed by global tooltip hiding on touch (`@media not (hover: hover)`). Add `aria-label` attributes to all action buttons for screen readers. Consider adding visible text labels on mobile for the most important actions.

### 5. Filter button tooltip (`WorkspaceSidebarVideoFilterControl.svelte:138`)

**Current**: `data-tooltip` on filter button, hover-only.
**Fix**: Add `aria-label`. If this button appears in a mobile context, pair it with a visible label or place it in a toolbar with labeled buttons.

### 6. Channel card delete hover overlay (`ChannelCard.svelte:193`)

**Current**: `opacity-0 group-hover:opacity-30` background overlay + hover-reveal delete button.
**Fix**: Addressed in [spec 03](./03-mobile-accessible-actions.md).

### 7. Chat message copy hover reveal (`ChatMessage.svelte:134`)

**Current**: `group-hover/copy:opacity-100`, invisible to touch.
**Fix**: Addressed in [spec 03](./03-mobile-accessible-actions.md).

### 8. Theme panel trigger (`ThemePanel.svelte`)

**Current**: `.theme-panel-trigger:hover` — hover-only focus ring.
**Fix**: Add `:focus-visible` styles alongside `:hover`. The trigger already accepts clicks/taps but has no visual feedback on touch.

## General Approach

For all `data-tooltip` usage on mobile:
1. Tooltips are already hidden on touch devices via `@media not (hover: hover)` in `app.css` — this is correct.
2. Ensure every tooltipped element has an `aria-label` for accessibility.
3. For critical controls (nav items, action buttons), add visible text labels on mobile.
4. For secondary controls where space is tight, ensure the element's function is clear from context or icon alone.

For all `group-hover:opacity-*` patterns:
1. Add `max-lg:opacity-100` to make elements always visible on mobile.
2. Or use `@media (hover: hover)` to scope the opacity-hiding to hover-capable devices only.

## Verification

- On a touch device, every interactive element is discoverable without hovering
- All tooltipped buttons have `aria-label` attributes
- Critical nav items have visible labels on mobile
- `bun run check`, `bun run test` pass
