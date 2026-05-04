# PRD: Content Loading Flash Fix

## Problem

Workspace and mini content panels load slowly enough that users see a visible "no content" state before the skeleton or real content paints. The empty branch is reachable while a fetch is still in flight, so the UI shows "No summaries", "Nothing to read yet", or a blank empty layout for one or more frames every time a user opens the app, switches channels, or pull-to-refreshes.

Two state files cause this:

- `frontend/src/lib/mini/mini-reader-state.svelte.ts:112` — `loading = $state(false)` at construction. The `+page.svelte` template at `frontend/src/routes/mini/+page.svelte:119` checks `mini.loading` first, then falls through to `mini.error`, then `!mini.activeSummary` → `MiniEmptyState` with `mini.emptyVariant`. Between mount and the first `loadReader()` setting `loading = true`, both `loading` and `activeSummary` are falsy and the empty branch wins.
- `frontend/src/lib/workspace/content-state.svelte.ts:62` — `loadingContent = $state(false)` with the same shape. `WorkspaceContentSurface.svelte` reads it at line 21 and gates skeletons on it. During pre-fetch and during refetch on selection change, the surface flickers between content states.

The empty/error/loading visual treatments are also too similar today. The skeleton in `MiniEmptyState.svelte` (`variant === "loading"`) shows generic blocks that do not match the article layout, which means even when the skeleton shows it does not feel like content is on its way.

## Goal

Eliminate the "no content" flash on initial load, channel switch, and refresh in both mini and workspace surfaces. Make the loading skeleton, empty state, and error state visually distinct so the user always knows which state they are in.

## Current Increment

Convert mini and workspace content loading to an explicit state machine, default to a non-empty initial state, and refresh the skeleton/empty/error visuals so all three are distinguishable.

This is increment 1 of a multi-step content loading roadmap (see Roadmap below). Streaming, prefetch, caching, and egress work are explicitly **not** part of this increment.

## Clear Deliverable

One PR that delivers all of:

1. `MiniReaderState.loading` (boolean) replaced with `status: "idle" | "loading" | "ready" | "empty" | "error"`. Initial value is `"loading"`. Empty UI only renders when `status === "empty"`.
2. `content-state.svelte.ts` loading flag converted to the same five-state enum. `WorkspaceContentSurface.svelte` and `WorkspaceContentPanel.svelte` updated to gate on the new state.
3. During refetch (channel switch, pull-to-refresh, retry), the previously rendered `activeSummary` (or workspace content) is held in place until the new payload arrives. Stale-while-revalidate at component level. State machine value during this period is `"loading"` with cached content still painted.
4. Mini route `+page.svelte` rewritten so the `{#if mini.loading}` chain becomes a single `{#if status === "..."}` switch with explicit branches for each state. No fall-through to empty during loading.
5. `MiniEmptyState.svelte` skeleton (`variant === "loading"`) restructured to match the real article shape: top meta bar, title placeholder, audio strip placeholder, paragraph blocks, action bar. Block widths and rhythm match a real summary.
6. `LoadingSkeleton.svelte` audited; if its shape no longer matches `WorkspaceContentSurface.svelte` final layout, it is updated in the same PR.
7. Stage label below or beside the skeleton, replacing the silent skeleton: `"Fetching summary…"` during initial fetch, `"Loading body…"` if/when tier hand-off lands in a future increment (placeholder copy lives in code now, used later).
8. Empty state and error state visually distinct from each other and from loading. Each has its own icon/illustration, label, and headline. Same `MiniEmptyState.svelte` component, three clearly different layouts.
9. Stale badge component (small "updating…" pill near title) wired up so future cache-then-revalidate work has the affordance ready. Badge shows whenever `status === "loading"` and cached content is being kept on screen.

## Non-Goals

- Tiered backend payload, NDJSON/SSE streaming, partial paint of body. Deferred to increment 4.
- IndexedDB / SW cache for last-viewed article. Deferred to increment 3 (the stale badge ships now, the cache populates later).
- ETag / `If-None-Match` round-trip. Deferred to increment 3.
- Mobile k+1 prefetch, desktop hover prefetch. Deferred to increments 5–6.
- Backend changes. This increment is frontend-only.
- New endpoints, new transport, new compression at rest, new sidebar batching. Deferred to increment 7.
- CloudFront or any CDN. Out of scope for the entire roadmap (no permanent free tier, cost prohibitive).
- Audio loading. Audio is not the current bottleneck.
- Sidebar redesign or sidebar empty/error states. Only the content surface in workspace and mini.

## Users or Actors

- **Mini reader user (mobile primary)**: opens `dastill-mini`, expects an article skeleton then content. Currently sees an empty-state copy first.
- **Workspace user (desktop primary)**: clicks a video in the sidebar, expects the content panel to show a skeleton then content. Currently sees a brief blank or empty state.
- **User refreshing or switching channel**: expects current article to remain visible until new one loads. Currently the panel goes empty mid-fetch.

## Requirements

### 1. State machine in `MiniReaderState`

- Replace `loading = $state(false)` with `status = $state<"idle" | "loading" | "ready" | "empty" | "error">("loading")`.
- Replace `error = $state<string | null>(null)` reads in templates with `status === "error"` checks; the message string itself stays as `errorMessage` for display.
- `loadReader()` transitions:
  - On entry: if `activeSummary` exists, keep it and set `status = "loading"` (refetch path). If no `activeSummary`, set `status = "loading"`.
  - On success with summaries: `status = "ready"`.
  - On success with zero summaries: `status = "empty"`.
  - On failure: `status = "error"`, populate `errorMessage`.
- `emptyVariant` derivation only consulted when `status === "empty"`.
- Existing tests in `frontend/tests/` that read `mini.loading` updated to read `mini.status`.

### 2. State machine in `content-state.svelte.ts`

- `loadingContent` boolean replaced with the same five-state enum exported as `contentStatus`.
- All five `loadingContent = false` and `loadingContent = true` mutations in `content-state.svelte.ts` rewritten to set the appropriate status.
- `WorkspaceContentSurface.svelte` line 21 prop renamed `loadingContent` → `contentStatus`. All gating expressions updated.
- `WorkspaceContentPanel.svelte` `loadingContent` derived value replaced with `contentStatus`. Props plumbed through to `WorkspaceContentSurface`, `WorkspaceVideoInfoPanel`, `WorkspaceHighlightsPanel`.
- `home-workspace.svelte.ts:258` derived value updated.
- `WorkspaceHomePage.svelte:127` prop pass-through updated.

### 3. Stale-while-revalidate at component level

- During `loadReader()` re-entry (e.g. `bypassCache: true` path at `mini-reader-state.svelte.ts:317`), do not null `reader`, `activeSummary`, or summary list until the new fetch resolves successfully.
- On fetch success, atomically swap to the new payload.
- On fetch failure, keep the prior payload painted, surface the error via a non-blocking toast (`ErrorToast.svelte`) rather than the full-pane error state. Full-pane error is only shown when there is no prior payload to fall back to.
- Same rule in `content-state.svelte.ts`: refetch holds prior content; failure with prior content surfaces a toast, not a panel-replacing error.

### 4. Mini route template rewrite

- `frontend/src/routes/mini/+page.svelte:118-150` rewritten to a single `{#if mini.status === "loading"}` / `{:else if mini.status === "error"}` / `{:else if mini.status === "empty"}` / `{:else}` switch.
- The current implicit fall-through (`{#if mini.loading}` → `{:else if mini.error}` → `{:else if !mini.activeSummary}`) is removed.
- `mini.activeSummary` is only consulted inside the `ready` branch.

### 5. Skeleton shape match

- `MiniEmptyState.svelte` `variant === "loading"` block (lines 17–30) restructured to mirror `MiniArticle.svelte` final layout:
  - Top meta row (channel badge + duration)
  - Title placeholder (two lines, second short)
  - Audio strip placeholder (full-width pill)
  - Paragraph blocks (3–4 lines each, varied widths, two paragraphs)
  - Bottom action bar placeholder
- `LoadingSkeleton.svelte` audited against `WorkspaceContentSurface.svelte` final layout (which renders summary header, audio player, body, highlights). If shape diverges, update.
- Skeleton block widths use the same rhythm as the real layout to avoid visual jump on hand-off.

### 6. Stage label

- Below (mobile) or beside (desktop) the skeleton hero block, render a small status label.
- Initial copy:
  - `"Loading…"` while `status === "loading"` and no cached content.
  - `"Updating…"` while `status === "loading"` and cached content is shown.
- The same label container is reused later for tiered hand-off ("Fetching summary…" → "Loading body…"). Ship the container and the two-state copy now.

### 7. Distinct empty/error/loading visuals

- `MiniEmptyState.svelte` empty branches keep current copy but each variant gets a distinct icon/illustration. `error` variant gets a different illustration than `no-subscriptions`, `all-read`, `no-summaries`.
- Loading skeleton has no icon; it is unmistakably a skeleton.
- Workspace error path (currently rendered via `LoadingSkeleton` with `contentStatus === "pending"` mode at `WorkspaceContentSurface.svelte:191-198`) gets a real error component with a retry button, not a styled skeleton.

### 8. Stale badge wiring

- Small badge component (e.g. `<StaleBadge />`) added next to the article title in `MiniArticle.svelte` and to the workspace content header in `WorkspaceContentSurface.svelte`.
- Visible whenever `status === "loading"` and cached content is being kept on screen.
- Hidden in all other states.
- Visual: subtle dot + "updating…" text, accessible via `aria-live="polite"`.

## Roadmap (future increments, not in this PR)

2. Skeleton + stage label polish pass after telemetry shows the new states in production.
3. Cache-then-revalidate with IndexedDB / SW + ETag/`If-None-Match` for content fetch (TTFC + egress win).
4. Tiered backend payload and NDJSON/SSE streaming so first chunk paints before tail arrives.
5. Mobile k+1 prefetch in `requestIdleCallback` with network-aware throttle and visibility cancel.
6. Desktop sidebar hover/focus prefetch with debounce, N=2 neighbor cap.
7. S3 compression at rest, libSQL-first sidebar meta, batched sibling meta endpoint.

Each future increment ships its own `.specs/<slug>/` pair.

## Risks and Open Questions

- **Risk**: Test files referencing `mini.loading` or `loadingContent` will break. Audit every test under `frontend/tests/` matching either name and update in the same PR. No grace period or deprecation shim.
- **Risk**: Holding prior content during refetch can mask a slow backend; the user might think nothing is happening. Mitigated by the stale badge and the "Updating…" stage label.
- **Risk**: Toast-on-refetch-failure is a behavior change. Today a refetch failure surfaces in the main pane via `mini.error`. Toast may be missed. Acceptable for this increment since prior content is still useful and the retry remains reachable through pull-to-refresh and explicit retry actions.
- **Open question**: Should the workspace error path also keep prior content + toast, or always replace the pane? Default for this increment: same rule as mini (keep prior, toast on refetch fail; full-pane error only when there is no prior content). Revisit if QA prefers per-surface differentiation.
- **Open question**: Should the stage label also expose a manual retry affordance during long fetches (>5s)? Default: no. Add only if telemetry shows long fetches.
- **Open question**: Where does the stale badge anchor in `WorkspaceContentSurface.svelte` when both summary and highlights tabs are active? Default: anchor next to the active tab's title only.
