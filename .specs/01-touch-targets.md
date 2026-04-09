# Spec 01: Touch Target Sizes

Priority: **P0**
Scope: All interactive elements across the app

## Problem

Nearly every icon button, toggle, and tappable control is below the 44x44px WCAG 2.5.5 minimum. On mobile, users mis-tap, double-tap, or avoid controls entirely.

## Requirement

Every interactive element (button, link, toggle, slider thumb) must have a minimum touch target of 44x44px on screens below 1024px. This can be achieved via:
- Increasing the element size directly
- Adding transparent padding that expands the hit area (`p-2` wrappers, `::after` pseudo-elements)
- Using `min-h-[44px] min-w-[44px]` on button wrappers

Desktop sizes can remain smaller where hover precision allows it.

## Files and Specific Changes

### Critical (24px targets)

| File | Line(s) | Current | Target |
|------|---------|---------|--------|
| `WorkspaceNavRail.svelte` | 148 | `h-6 w-6` (24px) collapse toggle | `h-11 w-11` (44px) or remove on mobile |
| `WorkspaceSidebarChannelControls.svelte` | 87, 108, 127, 167 | `h-6 w-6` (24px) action buttons | `h-11 w-11` (44px) |
| `WorkspaceSidebarVideoFilterControl.svelte` | 103 | `h-6 w-6` (24px) filter button | `h-11 w-11` (44px) |
| `ThemePanel.svelte` | CSS | 24px color swatches, 24px mode buttons | 44px min with spacing |
| `FeatureGuide.svelte` | CSS | 6px pip dots | 44px hit area with visual 6px dot |

### Severe (28-32px targets)

| File | Line(s) | Current | Target |
|------|---------|---------|--------|
| `WorkspaceSidebarCollapsedChannelRail.svelte` | 22 | `h-7 w-7` (28px) expand | `h-11 w-11` (44px) |
| `WorkspaceSidebarVideoRow.svelte` | row | `py-1.5` (~28px row) | `py-3` (~44px row) |
| `WorkspaceSidebarSelectedVideoList.svelte` | 74, 109 | `py-1.5` (28px) | `py-3` (44px) |
| `Toggle.svelte` | - | `h-7` (28px) | `h-11` (44px) |
| `ChatSidebar.svelte` | 70, 80, 159 | `h-7` (28px) buttons | `h-11` (44px) |
| `ChannelCard.svelte` | 124 | `h-7 w-7` (28px) delete | `h-11 w-11` (44px) |
| `WorkspaceHighlightsPanel.svelte` | 106, 130 | `h-8 w-8` (32px) | `h-11 w-11` (44px) |
| `AiStatusIndicator.svelte` | 37, 81 | `h-8 w-8` (32px) | `h-11 w-11` (44px) |
| `ContentEditor.svelte` | 134, 298 | `h-8 w-8` (32px) | `h-11 w-11` (44px) |
| `SearchResultsPopover.svelte` | 131 | `h-8 w-8` (32px) close | `h-11 w-11` (44px) |
| `ChatMessage.svelte` | 138 | `h-8 w-8` (32px) copy | `h-11 w-11` (44px) |
| `ChatInputControls.svelte` | 106, 125 | `h-8 w-8` (32px) | `h-11 w-11` (44px) |

### Below threshold (36-40px targets)

| File | Line(s) | Current | Target |
|------|---------|---------|--------|
| `WorkspaceContentMobileHeader.svelte` | 89 | `h-9 w-9` (36px) back | `h-11 w-11` (44px) |
| `ContentActionButton.svelte` | 38 | `h-9 w-9` (36px) ghost | `h-11 w-11` (44px) |
| `AddSourceDrawer.svelte` | 272 | `h-9 w-9` (36px) close | `h-11 w-11` (44px) |
| `SectionNavigation.svelte` | 37-43 | `h-9`/`h-10` (36-40px) | `h-11` (44px) |
| `MobileChannelGallery.svelte` | 230 | `h-9 w-9` (36px) add | `h-11 w-11` (44px) |
| `ContentActionButton.svelte` | 36 | `h-10 w-10` (40px) outlined | `h-11 w-11` (44px) |
| `WorkspaceSummaryAudioPlayer.svelte` | 258, 305 | `h-10 w-10` (40px) | `h-11 w-11` (44px) |
| `WorkspaceSummaryAudioPlayer.svelte` | 282, 343, 370 | `h-8` (32px) secondary | `h-11` (44px) |

### Audio player slider

| File | Line(s) | Current | Target |
|------|---------|---------|--------|
| `WorkspaceSummaryAudioPlayer.svelte` | CSS ~466-477 | 12px thumb | 44px touch target (visual thumb can stay small, use `::after` for hit area) |

## Implementation Approach

1. Create a shared CSS utility or Tailwind class for mobile touch targets:
   ```css
   @media (max-width: 1023px) {
     .touch-target { min-height: 44px; min-width: 44px; }
   }
   ```
2. Apply to all icon buttons. For buttons that must stay visually small, use a transparent `::after` pseudo-element to expand the hit area.
3. For list rows (video rows, sidebar items), increase `py-` padding on mobile only using `max-lg:py-3`.
4. For the audio slider thumb, use the CSS `::after` approach to keep the thumb visually small but tappable.

## Verification

- On a real phone or Chrome DevTools mobile emulator (375px width), every button and control should be tappable on the first attempt without mis-tapping adjacent elements.
- Run `bun run check` and `bun run test` after changes.
