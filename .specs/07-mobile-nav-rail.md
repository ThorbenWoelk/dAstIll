# Spec 07: Mobile Nav Rail Rework

Priority: **P1**
Scope: `WorkspaceNavRail.svelte`, `WorkspaceShell.svelte`

## Problem

On mobile (< 1024px), the nav rail is force-collapsed to 52px (icon-only) and always visible on the left edge. This has several issues:

1. **Wastes 52px of horizontal space** on a 375px screen — that's 14% of the viewport width dedicated to navigation that could be in a bottom bar.
2. **Labels are hover-only** via `data-tooltip` — touch users see unlabeled icons.
3. **Collapse/expand toggle is hidden** on mobile (`hidden lg:inline-flex`), so users are stuck with the collapsed state.
4. **User menu** at the bottom of the rail opens a popup that may overflow the viewport (`w-72` = 288px, positioned at `absolute bottom-full` from a 52px-wide container).

## Requirement

Once the mobile bottom nav (spec 02) is implemented, **hide the nav rail entirely on mobile**:

1. Add `hidden lg:flex` to the `WorkspaceNavRail` root element (or its parent in `WorkspaceShell`)
2. Remove the `navWidth = NAV_MIN` mobile override in `WorkspaceShell.svelte:73`
3. Reclaim the full viewport width for content on mobile
4. Move the user menu trigger to the mobile bottom nav bar (rightmost icon or a profile avatar)

## If Bottom Nav Is Not Yet Built

As an interim improvement without the bottom nav:

1. Keep the rail but add visible text labels below each icon (vertically stacked, `flex-col items-center`)
2. Increase touch target size of nav icons to 44px
3. Fix the user menu popup positioning to stay within viewport bounds on narrow screens (`max-w-[calc(100vw-52px-16px)]`)

## Files to Modify

| File | Change |
|------|--------|
| `WorkspaceNavRail.svelte` | `hidden lg:flex` on root, or add labeled icons |
| `WorkspaceShell.svelte` | Remove mobile `navWidth` override; remove left margin on mobile |
| `WorkspaceUserMenu.svelte` | Fix popup overflow; add mobile trigger in bottom nav |

## Verification

- On mobile, nav rail is either hidden (with bottom nav) or has labeled, tappable icons
- Content area uses full viewport width on mobile
- User menu is accessible and doesn't overflow viewport
- `bun run check`, `bun run test` pass
