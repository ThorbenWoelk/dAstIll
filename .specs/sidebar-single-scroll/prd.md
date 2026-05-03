# PRD: Sidebar Single-Scroll Channel Preview

## Problem

On desktop, the workspace sidebar has two nested vertical scroll containers when a channel is expanded in `per_channel_preview` mode:

- Outer scroller: `WorkspaceSidebar.svelte:561` (`overflow-y-auto` on the channel list).
- Inner scroller: `WorkspaceSidebarPreviewChannelContent.svelte:59` (`max-h-[21rem] overflow-y-auto` when the collection is in `paged` mode).

Trackpad and wheel input is ambiguous near the boundary. Users hit the inner cap, expect the outer list to scroll, and get nothing until they leave the inner area. Reaching channels below an expanded channel requires moving the cursor outside the preview before scrolling. The interaction is unintuitive on desktop.

## Goal

The desktop sidebar has one scroll surface. Expanding a channel in `per_channel_preview` mode never introduces a nested scroll container. Wheel and trackpad input behave the same anywhere in the sidebar.

## Current Increment

Remove the inner scroll cap on the inline channel preview, keep the channel row pinned while its preview scrolls past, and ensure only one channel preview is expanded at a time.

## Clear Deliverable

In `per_channel_preview` mode (Workspace home and Highlights sidebar):

1. The expanded preview list flows directly into the outer sidebar scroller. No inner `overflow-y-auto` and no `max-h` cap on the preview container.
2. The expanded channel row uses `position: sticky; top: 0` against the outer sidebar scroller, so the channel header stays visible while its preview scrolls past.
3. Expanding a channel into the full paged list collapses any other channel that was in paged mode. At most one channel has `loadedMode === "paged"` at any time. Other channels keep showing their normal 5-video preview snippets.
4. The "Load more" button at the bottom of the paged collection remains. Infinite scroll inside a capped container is removed along with the cap.
5. Virtualization that previously read the inner container's `scrollTop` is updated to read the outer sidebar scroller's `scrollTop`, or the virtualization threshold is adjusted so virtualization is only triggered by the outer scroll position. Performance for large paged collections (hundreds of items) must not regress.
6. The inline "Loading channels" status row above the channel list is removed. Initial load already shows a skeleton list; the secondary status row is redundant.
7. Collapsing a paged channel via its sticky chevron anchors the channel row back to the top of the outer sidebar scroller (smooth `scrollTo`). Without this, the row vanishes above the viewport because the document shrinks while outer `scrollTop` stays put.

## Non-Goals

- Mobile sidebar behavior. Mobile uses a different overlay (`WorkspaceMobileBrowseOverlay.svelte`) and is out of scope for this increment.
- The `selected_channel` mode (legacy single-channel selected list). Only `per_channel_preview` is being changed.
- Moving the expanded list out of the sidebar into the center content pane (master-detail). Considered as an alternative; not chosen for this increment.
- Reworking sort, drag-and-drop, filters, or the add-source flow.
- Changing the per-channel preview API, snapshot loader, or pagination semantics.

## Users or Actors

- **Desktop users browsing channels**: scroll the sidebar with trackpad or wheel and expect a single, continuous scroll target.
- **Power users with many channels**: expand a channel, scan its videos, and continue scrolling to the next channel without cursor gymnastics.

## Requirements

### 1. Remove the inner scroll cap

- In `WorkspaceSidebarPreviewChannelContent.svelte`, drop `max-h-[21rem] overflow-y-auto pb-1 pr-1 [overscroll-behavior-y:contain]` from the `paged` branch. The container becomes a plain block that lets its rows extend the outer scroll height.
- Remove the `onscroll={onCollectionScroll}` handler from this container; scroll position is no longer owned here.

### 2. Sticky channel row while preview scrolls

- In `WorkspaceSidebarChannelRow.svelte`, extend the existing `sticky top-0 z-10 bg-[var(--surface)]` wrapper (currently applied only when `!isPreviewMode && isExpanded`) to also apply when `isPreviewMode && isExpanded`.
- Sticky offset is `top: 0` against the outer sidebar scroller (`WorkspaceSidebar.svelte:561`). Confirm no parent between the row and the scroller has `overflow: hidden`, `transform`, or `filter` that would break sticky behavior.
- Background must be opaque (`bg-[var(--surface)]`) so videos scrolling underneath are not visible through the sticky header.

### 3. Single-expand

- In `sidebar-preview-controller.svelte.ts`, change `toggleChannelVideoCollection` so expanding channel B collapses any other channel A whose `expanded === true` in `channelVideoCollections`. Collapse uses the existing `setPreviewChannelExpanded(channelId, false)` path.
- Selecting a channel via `handlePerChannelPreviewSelect` (which routes to `onOpenChannelOverview` or `onSelectChannel`) does not by itself force collapse; only explicit expand toggles do. Existing auto-expand on selection paths (`current.expanded = true` at controller line ~421) must also collapse siblings.
- Persisted preview session state (`resolveSidebarPreviewSessionKey`) must survive a reload with at most one expanded channel.

### 4. Keep "Load more"

- The existing `Load more` button branch in `WorkspaceSidebarPreviewChannelContent.svelte` stays.
- No new infinite-scroll trigger is added against the outer scroller in this increment. Loading more videos remains an explicit user action.

### 5. Virtualization continues to work

- The current virtualization in `resolveRenderedCollectionVideos` reads `collection.scrollTop`, which is fed by the inner `onCollectionScroll` handler. After removing the inner scroller, virtualization must either:
  - Read `scrollTop` from the outer sidebar scroller (`WorkspaceSidebar.svelte:561`) via a shared ref, translated to per-collection coordinates by the row's offset within the scroller, or
  - Be relaxed for paged collections under a higher threshold such that the visible window is rendered fully when not virtualized.
- Implementation choice is left to engineering, but the chosen approach must be covered by the existing `sidebar-preview-controller.test.ts` patterns or a new equivalent.

## Risks and Open Questions

- **Risk**: Sticky breaks if any ancestor of `WorkspaceSidebarChannelRow` between it and the outer scroller introduces `overflow: hidden`, `transform`, `filter`, or `contain`. Audit before relying on sticky.
- **Risk**: An expanded channel with a long paged list pushes other channels far down the sidebar. Single-expand mitigates this. Acceptable per current UX direction.
- **Risk**: Virtualization rewrite could regress scroll performance on very large channels. Benchmark with the largest available channel snapshot before merge.
- **Open question**: Should expanding a channel auto-scroll the outer sidebar so the sticky header lands at the top of the viewport? Default is no; revisit after implementation review.
- **Open question**: When the user clicks the sticky header to collapse, should the outer scroll snap back to that channel's row, or stay where it is? Default is stay.
