# Test Spec: Content Loading Flash Fix

## Acceptance Criteria

- [ ] On initial mount of `/mini`, the empty-state copy ("No subscriptions", "Nothing to read yet", "No summaries", "You're all caught up") never renders before the first `loadReader()` resolves.
- [ ] On initial mount of the workspace content panel, the empty/error visuals never render before the first content fetch resolves.
- [ ] During channel switch in mini and during selection change in workspace, the previously rendered article/content remains painted until the new payload arrives.
- [ ] During pull-to-refresh in mini, the current article remains painted; the stale badge is visible while the refetch is in flight.
- [ ] On refetch failure when prior content exists, prior content stays on screen and an error toast appears. The full-pane error state does not replace prior content.
- [ ] On first-load failure (no prior content), the full-pane error state appears with a retry affordance.
- [ ] Loading skeleton, empty state, and error state are visually distinct: each has its own iconography or shape, and a screenshot diff between any two would not be confusable.
- [ ] The skeleton in `MiniEmptyState.svelte` `variant === "loading"` mirrors the real article layout: meta row, title, audio strip, paragraph blocks, action bar.
- [ ] The skeleton block widths match the real article rhythm closely enough that hand-off from skeleton to content does not visibly reflow major elements.
- [ ] The stage label reads `"Loading…"` when no cached content is shown and `"Updating…"` when cached content is shown.
- [ ] The `<StaleBadge />` is visible in mini and workspace content headers whenever `status === "loading"` and cached content is on screen, and hidden in all other states.
- [ ] No template anywhere reads `mini.loading` or `loadingContent` after this PR. All gating uses `mini.status` and `contentStatus`.

## Proof for the Current Increment

### Automated Checks

**Unit (`frontend/tests/mini-reader-state.test.ts`, new or extended)**
- New `MiniReaderState()` initializes with `status === "loading"`, not `"idle"` or `"ready"`.
- `loadReader()` happy path: `loading` → `ready` when summaries exist, `loading` → `empty` when zero summaries, `loading` → `error` on fetch rejection.
- `loadReader({ bypassCache: true })` while `activeSummary` exists: `status` becomes `"loading"`, `activeSummary` retained, `reader` retained.
- On refetch success with new data, `activeSummary` swaps atomically (single tick) to the new payload.
- On refetch failure with prior `activeSummary`, prior `activeSummary` is retained, `errorMessage` is set, and the route layer (mocked) receives an error toast event.
- On first-load failure (no prior `activeSummary`), `status === "error"`, `errorMessage` is set, no toast event.

**Unit (`frontend/tests/workspace-content-state.test.ts` or extension of existing)**
- Same five-state contract as mini.
- Selection change (from video A to video B) keeps A's content painted until B's fetch resolves.
- Selection change failure with prior A keeps A on screen; toast event fires.

**Component (`frontend/tests/workspace-content-panel.test.ts`)**
- During `contentStatus === "loading"` with no prior content, only the skeleton renders.
- During `contentStatus === "loading"` with prior content, prior content + stale badge render; skeleton does not render.
- During `contentStatus === "empty"`, only the empty visual renders. The empty visual is not the loading skeleton.
- During `contentStatus === "error"` with no prior content, the error visual renders with a retry button.

**Component (mini route, `frontend/tests/mobile-navigation.test.ts` or new)**
- Initial render of `/mini` before fetch resolution shows skeleton, never empty copy.
- Channel switch keeps prior article visible during the fetch.

**Visual / DOM**
- DOM snapshot test for the skeleton ensures presence of: `.skel-meta`, `.skel-title`, `.skel-audio`, ≥ 2 paragraph block groups, `.skel-action`.
- Skeleton container does not contain `aria-live` text that reads as empty content.

### Manual Checks

- Cold load of `/mini` on a throttled mobile profile (Chrome DevTools "Slow 3G"): user sees skeleton, then content. No empty-state copy at any point.
- Cold load of workspace home, then click a sidebar video: skeleton appears in the content panel. No "Select a video" or empty-state copy in between.
- Switch between channels in mini: prior article visible until new article paints. Stale badge shows during the switch.
- Pull-to-refresh in mini with airplane mode after pull starts: prior article stays, error toast appears. No full-pane error.
- Pull-to-refresh in mini fresh tab with airplane mode: full-pane error with retry button.
- Empty filter case (toggle "Unread only" with no unread items): empty state renders only after fetch completes; visual is unmistakably the empty state, not the skeleton.
- Workspace selection change with backend artificially delayed: prior content held, stale badge visible, hand-off without flash.

### E2E (`frontend/e2e/`)

- New spec `content-loading.spec.ts` (or extension of existing mini/workspace specs):
  - Throttle network to "Slow 3G" via Playwright route handler.
  - Load `/mini`. Capture frame-by-frame between navigation start and first article paint. Assert no frame contains text matching `"Nothing to read yet"`, `"No subscriptions"`, `"No summaries"`, `"You're all caught up"`, `"Reader unavailable"`.
  - Switch channel. Assert previous article DOM nodes remain present in the DOM during the network round-trip.
  - Force fetch failure on second channel via route interception. Assert the previous article remains, an error toast is visible, and the full-pane error is not.
- Workspace equivalent: load workspace home, click sidebar video A, click sidebar video B, assert A's content stays painted until B resolves.

## Edge Cases

- **No subscriptions yet**: first-load with empty subscription list. After fetch resolves, `status === "empty"` and `emptyVariant === "no-subscriptions"`. Skeleton is not visible after resolution.
- **Authentication failure on first load**: `status === "error"`, full-pane error with retry. Retry path goes through `mini.loadReader(mini.selectedChannelId)`.
- **Authentication failure on refetch**: prior content retained, toast surfaced.
- **Filter toggle with cached unread list**: status flips to `"loading"` only if a refetch is needed; otherwise the filter result paints without skeleton.
- **Reader.channels populated but no summaries in selected channel**: `status === "empty"`, `emptyVariant === "no-summaries"`. Skeleton must not race the empty.
- **Concurrent `loadReader()` calls (rapid channel switching)**: only the last fetch determines final state. Earlier in-flight fetches are ignored or aborted.
- **Slow backend (>5s)**: skeleton + "Loading…" or "Updating…" label keep showing. No timeout-induced empty state.
- **Pull-to-refresh during an in-flight refetch**: second refetch supersedes the first; prior content stays painted across both.
- **Workspace tab switch (summary ↔ highlights) during a content fetch**: in-flight fetch survives the tab switch; content paints in both tabs once resolved.

## Observability or Failure Signals

- Unit + component tests fail if any template gates on `mini.loading` or `loadingContent` after refactor.
- Lint rule (or grep gate in CI) for `mini\.loading|loadingContent` returning zero matches in `frontend/src/`.
- E2E frame assertion fails if any forbidden empty-state string appears between navigation start and first content paint.
- Console must remain free of Svelte warnings about removed reactive bindings.
- Lighthouse / Web Vitals: CLS at `/mini` and workspace home does not regress versus pre-change baseline (skeleton-to-content hand-off should reduce CLS, not increase it).
- Screenshot diff suite: skeleton, empty (each variant), and error renders are kept as goldens; visual regressions are blocking.
