# Mobile UI Audit Summary

Status: **Not mobile-ready**

The app has a structural mobile foundation (viewport inset tracking, swipe gestures, overlay drawers, `lg:` breakpoint toggling) but falls short of a usable mobile experience. The problems are systemic, not cosmetic.

## Critical Issues (Blocking)

1. **Touch targets are too small everywhere.** Virtually every icon button is 24-40px. WCAG 2.5.5 minimum is 44x44px. Audio player slider thumb is 12px. Theme color swatches are 24px. Sidebar controls are 24px. This makes the app difficult to use with fingers.

2. **Key actions are inaccessible on mobile.** Channel deletion is `max-lg:hidden` with no alternative. Chat conversation rename/delete buttons are hover-reveal only with no touch fallback. These are functional regressions, not design choices.

3. **iOS auto-zoom on input focus.** Six input fields use font sizes below 16px (12-15px), causing Safari to zoom the page on focus. This breaks layout and is disorienting.

4. **No unified mobile bottom navigation.** The design system specifies a "shared app-level bottom tab bar for Workspace | Queue | Highlights | Settings." A `mobileBottomBar` store exists but no rendering component was found. Users navigate via a 52px icon-only rail that has no labels on touch devices.

5. **Nav rail labels are hover-only.** When collapsed to 52px on mobile, section labels depend on `data-tooltip` hover. Touch users see unlabeled icons with no way to preview what they do.

## Significant Issues

6. **Hover-reveal patterns with no touch alternative.** Chat message copy button, sidebar video row preview, channel card delete overlay all require mouse hover.

7. **Font sizes too small.** 8px pipeline status text, 9px metadata labels, 10px load-more buttons. Illegible on mobile without zooming.

8. **Feature guide tour card** uses fixed JS pixel positioning that ignores `visualViewport` changes. Navigation dots are 6px (untappable). Buttons are under 44px.

9. **Chat input controls stack vertically** below 640px, eating viewport space. The mobile chat experience needs a compact input layout.

10. **No intermediate breakpoints.** Only `lg: 1024px` is used structurally. No `sm:` or `md:` breakpoints for tablets or large phones. The app is either "full desktop" or "narrow phone."

## What Works

- Visual viewport tracking (`MobileViewportInset`) for keyboard handling
- Swipe-back gesture from left edge
- Horizontal tab-swipe on content panel
- Full-screen overlay/drawer pattern for browse and sidebar
- Safe-area-inset handling for notch/home-indicator
- Service worker thumbnail caching for mobile gallery
- PWA manifest and homescreen-ready configuration
- Tooltip hiding on touch devices (`@media not (hover: hover)`)
- Drag-and-drop correctly disabled on mobile

## Spec Index

| Spec | Scope | Priority |
|------|-------|----------|
| [01-touch-targets](./01-touch-targets.md) | Resize all interactive elements to 44px minimum | P0 |
| [02-mobile-bottom-nav](./02-mobile-bottom-nav.md) | Build the unified bottom tab bar | P0 |
| [03-mobile-accessible-actions](./03-mobile-accessible-actions.md) | Restore missing mobile actions (delete, rename, etc.) | P0 |
| [04-ios-input-zoom](./04-ios-input-zoom.md) | Fix input font sizes to prevent iOS auto-zoom | P0 |
| [05-hover-to-touch](./05-hover-to-touch.md) | Replace hover-only patterns with touch alternatives | P1 |
| [06-mobile-typography](./06-mobile-typography.md) | Fix illegible font sizes on mobile | P1 |
| [07-mobile-nav-rail](./07-mobile-nav-rail.md) | Rethink nav rail for mobile | P1 |
| [08-mobile-chat](./08-mobile-chat.md) | Optimize chat page for mobile | P1 |
| [09-feature-guide-mobile](./09-feature-guide-mobile.md) | Fix tour card positioning and touch targets | P2 |
| [10-responsive-breakpoints](./10-responsive-breakpoints.md) | Add intermediate breakpoints for tablets | P2 |
