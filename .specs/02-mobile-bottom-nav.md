# Spec 02: Mobile Bottom Navigation Bar

Priority: **P0**
Scope: New component, all page routes

## Problem

The design system (design.md) specifies a "shared app-level bottom tab bar for Workspace | Queue | Highlights | Settings" but no rendering component exists. The `mobileBottomBar` store in `mobile-navigation/mobileBottomBar.ts` tracks state (`sections`, `sectionsWithVideoFilter`, `videoActions`, `hidden`) but nothing consumes it visually.

On mobile, the only navigation is a 52px icon-only rail on the left edge with hover-only labels. This is not how mobile apps navigate. Users expect a bottom tab bar.

## Requirement

Build a `MobileBottomNav.svelte` component that:

1. Renders as a fixed bottom bar on screens below 1024px (`lg:hidden`)
2. Shows 4-5 section tabs: **Workspace**, **Queue**, **Highlights**, **Chat**, and optionally **Vocabulary** or **Settings**
3. Each tab shows an icon + text label (always visible, not hover-dependent)
4. Highlights the active section based on current route
5. Respects `safe-area-inset-bottom` for home indicator clearance
6. Respects `--mobile-viewport-offset-bottom` for keyboard visibility (hides or shifts when keyboard is open)
7. Consumes the existing `mobileBottomBar` store to show/hide contextual controls
8. Uses opaque background per the design system (no backdrop-filter, no transparency)

## Design

```
+-----------------------------------------------------+
|  [icon]     [icon]     [icon]     [icon]     [icon]  |
| Workspace   Queue   Highlights   Chat      Vocab    |
+-----------------------------------------------------+
     ^active (accent color + indicator)
```

- Height: 56-64px (content) + `env(safe-area-inset-bottom)` padding
- Background: `var(--surface)` solid
- Border: `border-t border-[var(--border-soft)]`
- Active tab: `var(--accent-strong)` icon + label, `var(--accent-soft)` background pill
- Inactive tab: `var(--soft-foreground)` icon + label
- Touch targets: each tab must be at least 44px wide and 44px tall
- Label font: 10px uppercase bold tracking-wide (matches design system UI label spec)
- Icons: reuse existing stroke icon components or add new ones to `icons/`
- Transitions: 150ms color transition, no layout shift

## Architecture

### New files
- `frontend/src/lib/components/mobile/MobileBottomNav.svelte`

### Modified files
- `frontend/src/routes/+layout.svelte` — mount `MobileBottomNav` inside the root flex container, after the `children()` slot
- `frontend/src/lib/components/workspace/WorkspaceNavRail.svelte` — hide entirely on mobile (`hidden lg:flex`) since the bottom nav replaces it
- `frontend/src/lib/components/workspace/WorkspaceShell.svelte` — remove the 52px nav rail width reservation on mobile; update bottom padding to account for bottom nav height
- `frontend/src/app.css` — update `--mobile-bottom-stack-height` to include the bottom nav height (e.g., `calc(var(--mobile-viewport-offset-bottom) + 64px)`)

### Store integration
- Read `mobileBottomBar` store in `MobileBottomNav`
- When state is `hidden`, hide the bar (e.g., during full-screen overlays)
- When state is `sectionsWithVideoFilter`, show filter control inline or as an extra row
- When state is `videoActions`, show contextual video action buttons instead of section tabs

### Route detection
- Use `resolveCurrentSectionFromPathname.ts` (already exists) to determine active section
- Subscribe to SvelteKit `page` store for reactive route updates

## Impact on Existing Mobile Layout

- The nav rail's 52px left margin disappears on mobile, giving content the full viewport width
- All `mobile-bottom-stack-padding` values must account for the new nav bar height
- Overlay components (`z-[70]`, `z-[80]`) must render above the bottom nav (`z-[60]` for the nav)
- The `SectionNavigation` floating pill (currently `mobile-bottom-nav-offset`) becomes redundant and should be removed on pages that use the bottom nav

## Verification

- Bottom nav visible on all pages at < 1024px viewport
- Active section correctly highlighted on each route
- Tapping a tab navigates to the correct page
- Bar hides when keyboard is open (via `--mobile-viewport-offset-bottom`)
- Bar respects safe-area-inset on iPhone notch devices
- No layout shift when navigating between pages
- `bun run check`, `bun run lint`, `bun run test` pass
