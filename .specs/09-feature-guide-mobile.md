# Spec 09: Feature Guide Mobile Fix

Priority: **P2**
Scope: `FeatureGuide.svelte`

## Problems

1. **Fixed positioning ignores viewport changes** — tour card uses `position: fixed` with JS-computed `top`/`left` from `getBoundingClientRect`. When the software keyboard opens or browser chrome shows/hides, the visual viewport shifts and the card may be obscured or mispositioned. The component does not listen to `visualViewport` resize events.

2. **Navigation dots are 6px** (`.tour-pip`) — far too small to tap. Users cannot jump between tour steps on mobile.

3. **Navigation buttons are below 44px** — `.tour-close` (28px), `.tour-nav-back` and `.tour-nav-next` (~32px height).

4. **Tour card may overflow narrow screens** — at `max-width: 340px` with `padding: 24px`, the card fits a 375px screen but has only ~17px margin on each side. On smaller devices (320px iPhone SE) it overflows.

## Requirements

### Positioning
- Listen to `window.visualViewport` resize and scroll events (same pattern as `MobileViewportInset.svelte`) and reposition the tour card when the viewport changes.
- On mobile (< 1024px), prefer centering the card in the visible viewport area rather than anchoring to a highlight target. The highlight target may be scrolled off-screen or obscured by the keyboard.
- Add `max-width: min(340px, calc(100vw - 32px))` to prevent overflow on narrow devices.

### Touch targets
- Increase `.tour-pip` hit area to 44px (visual dot stays 6px, but wrap in a 44px tappable container).
- Increase `.tour-close` to 44px.
- Increase `.tour-nav-back` and `.tour-nav-next` to 44px height.

### Mobile layout
- On mobile, at the `@media (max-width: 480px)` breakpoint that already exists in the component, position the card at the bottom of the viewport (like a bottom sheet) instead of floating near the target element.

## Files to Modify

| File | Change |
|------|--------|
| `FeatureGuide.svelte` | Viewport listener, mobile positioning, touch target sizes |

## Verification

- Tour card stays visible and properly positioned when keyboard opens/closes on mobile
- All navigation controls are tappable (44px minimum)
- Card does not overflow on 320px-wide screens
- `bun run check`, `bun run test` pass
