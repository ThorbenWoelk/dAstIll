# Test Spec: Sidebar Single-Scroll Channel Preview

## Acceptance Criteria

- [ ] In `per_channel_preview` mode on desktop (>= 1024px), expanding a channel does not introduce a second scroll container inside the sidebar.
- [ ] The outer sidebar scroller (`WorkspaceSidebar.svelte:561`) is the only `overflow-y: auto | scroll` ancestor between the expanded preview list and the viewport.
- [ ] The expanded channel row stays pinned at the top of the visible sidebar area while its preview list scrolls past, on both `selected_channel` and `per_channel_preview` modes.
- [ ] At any time, at most one channel preview has `expanded === true` in `channelVideoCollections`.
- [ ] Expanding channel B while channel A is expanded collapses A.
- [ ] The `Load more` button still appears at the bottom of a paged collection when `hasMore && !loadingMore`.
- [ ] No regression in virtualized rendering for paged collections above `VIRTUALIZATION_THRESHOLD`.

## Proof for the Current Increment

### Automated Checks

**Unit (`frontend/tests/sidebar-preview-controller.test.ts` or new file)**
- Expanding channel B via `toggleChannelVideoCollection` collapses channel A when A was previously expanded.
- Auto-expand on selection (`current.expanded = true` path) collapses siblings.
- Session restore from `resolveSidebarPreviewSessionKey` never produces more than one expanded collection.
- Virtualization output (`resolveRenderedCollectionVideos`) returns the expected window when fed the outer-scroller `scrollTop` (or matches the chosen post-rewrite contract).

**Component (`frontend/tests/workspace-sidebar.test.ts`)**
- In `per_channel_preview` mode, the rendered DOM for the expanded preview container does not have `overflow-y-auto` or `max-h-[21rem]` classes on the paged branch.
- The expanded channel row wrapper has `sticky top-0 z-10 bg-[var(--surface)]` classes when `isPreviewMode && isExpanded`.

**E2E (`frontend/e2e/workspace.spec.ts` or new `sidebar-single-scroll.spec.ts`)**
- At viewport 1280x800, expand a channel with > 20 videos:
  - Assert there is exactly one ancestor of the preview list with computed `overflow-y` of `auto` or `scroll` between the list and the document.
  - Wheel-scroll inside the expanded preview area, verify the outer sidebar scroller advances (its `scrollTop` increases) on the same gesture.
  - Verify the channel row's bounding box top stays at the top of the sidebar scroll container while the preview rows below it scroll out of view.
- Expand channel A, then expand channel B. Assert A's preview is no longer in the DOM and only B is expanded.

### Manual Checks

- Trackpad two-finger scroll inside the expanded preview continues to scroll the outer sidebar past the bottom of the preview without lifting fingers.
- Wheel scroll over the expanded preview moves the outer sidebar in a single continuous motion.
- Sticky header reads correctly against a scrolled background (no transparency bleed, no z-index issues with drag indicators or popovers).
- Keyboard `Tab` and arrow navigation through video rows still scrolls them into view via the existing `scrollIntoViewWhenSelected` action.
- `Load more` button is reachable by scrolling the outer sidebar to the bottom of the expanded preview.

## Edge Cases

- **Channel with zero videos**: empty caption renders, no scroll change, sticky header remains correct.
- **Channel still in `loadingInitial`**: skeleton rows render under the sticky header, no inner scroll.
- **Largest available channel (hundreds of videos, paged)**: virtualization remains active, no jank when scrolling the outer sidebar across the expanded range.
- **Drag-and-drop reorder while a channel is expanded**: drop indicators and the dragged ghost render correctly with the sticky header in place.
- **Session restore with a previously expanded channel**: that channel re-expands on load; no second channel auto-expands.
- **`OTHERS_CHANNEL_ID` virtual channel**: still excluded from expand toggling per `isVirtualChannel` guard.

## Observability or Failure Signals

- DOM check in E2E: count of `overflow-y: auto | scroll` ancestors between the expanded preview list and `body` must equal 1.
- Console must remain free of Svelte warnings about removed bindings or invalid sticky context.
- Lighthouse CLS at the workspace home should not regress versus pre-change baseline.
- If virtualization is rewritten to read the outer scroller, a regression check on `resolveRenderedCollectionVideos` ensures the rendered window still includes the selected video when present.
