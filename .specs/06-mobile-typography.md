# Spec 06: Mobile Typography Fixes

Priority: **P1**
Scope: Illegibly small text across components

## Problem

Several UI elements use font sizes that are unreadable on mobile screens, especially on non-retina or low-DPI Android devices. While the design system permits 10-11px for "UI Labels / Tabs / Tooltips," some usages go below even that floor.

## Affected Text

### Below 10px (illegible, must fix)

| File | Line | Current | Content | Fix |
|------|------|---------|---------|-----|
| `VideoCard.svelte` | 57 | `text-[8px]` | Pipeline status icons | `text-[10px]` minimum, or use icon-only status |
| `WorkspaceSummaryMeta.svelte` | CSS ~109 | `font-size: 9px` | Metadata labels | `font-size: 10px` or `11px` |

### At 10px (borderline, review for mobile)

| File | Line | Current | Content | Recommendation |
|------|------|---------|---------|----------------|
| `WorkspaceSidebarSelectedVideoList.svelte` | 109 | `text-[10px]` | "Load More" button | `text-[11px]` — this is an actionable button, not a label |
| `WorkspaceSidebarPreviewChannelContent.svelte` | 130 | `text-[10px]` | "Load more" button | Same |

### At 12px (tight on low-DPI mobile)

| File | Line | Current | Content | Recommendation |
|------|------|---------|---------|----------------|
| `WorkspaceSidebarVideoRow.svelte` | row text | `text-[12px]` | Video titles | Acceptable on retina. Consider `text-[13px]` on mobile |
| `QueueContentPanel.svelte` | 315 | `text-[12px]` | Date input label | `text-base` on mobile (also fixes iOS zoom, see spec 04) |

## Rules Going Forward

1. **Minimum body text**: 13px on mobile, 12px on desktop.
2. **Minimum label text**: 10px, only for uppercase tracking-wide labels per the design system.
3. **Minimum button/link text**: 11px — interactive text must be readable without effort.
4. **Never use 8px or 9px** for any text that a user needs to read.

## Verification

- Visual review on a 375px-wide viewport: all text readable without squinting
- `bun run check`, `bun run test` pass
