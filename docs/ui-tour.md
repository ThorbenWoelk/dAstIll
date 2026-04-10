---
title: UI Tour
pageClass: ui-tour-page
---

# UI Tour

<div class="tour-update-banner">
  <p class="tour-fact-label">Latest update</p>
  <p>
    April 10, 2026. This page now documents the current <strong>mobile web</strong> UI first and uses refreshed screenshots from that layout.
  </p>
</div>

<div class="tour-intro">
  <p class="tour-eyebrow">Mobile Web First</p>
  <p class="tour-lede">
    dAstIll works best as a short loop: browse followed sources, open a recent item, read the
    summary or transcript, then switch to queue or chat when you need more detail.
  </p>
  <p class="tour-route-line">
    <strong>Core routes:</strong> Workspace, Queue, Highlights, Vocabulary, Chat, Docs.
  </p>
</div>

<div class="tour-facts">
  <div class="tour-fact-row">
    <p class="tour-fact-label">Primary format</p>
    <p>The screenshots below use the current phone-sized web layout. Desktop still exists, but this tour is intentionally mobile-led.</p>
  </div>
  <div class="tour-fact-row">
    <p class="tour-fact-label">Primary loop</p>
    <p>Browse, read, check processing status, ask questions. The routes are separate, but they keep the same content and state model.</p>
  </div>
  <div class="tour-fact-row">
    <p class="tour-fact-label">Signed-out experience</p>
    <p>Anonymous browsing and quota-limited chat remain available even before sign-in.</p>
  </div>
  <div class="tour-fact-row">
    <p class="tour-fact-label">Guide entry point</p>
    <p>The Guide button in the navigation rail reopens the built-in walkthrough from inside the workspace.</p>
  </div>
</div>

## Surface Map

<div class="tour-surface-list">
  <article class="tour-surface-row">
    <p class="tour-surface-label">Workspace</p>
    <p><strong>Main view.</strong> Browse sources, scan synced items, and switch the selected item between info, summary, highlights, and transcript-like text.</p>
  </article>
  <article class="tour-surface-row">
    <p class="tour-surface-label">Queue</p>
    <p><strong>Processing status.</strong> See transcript extraction, summary generation, failures, and backfill depth without leaving the main layout.</p>
  </article>
  <article class="tour-surface-row">
    <p class="tour-surface-label">Channel overview</p>
    <p><strong>Source-level view.</strong> Open a dedicated channel route to inspect one source, manage sync depth, and review that channel without the full reader layout.</p>
  </article>
  <article class="tour-surface-row">
    <p class="tour-surface-label">Chat</p>
    <p><strong>Library chat.</strong> Ask questions across the same library, switch models per conversation, and turn on deep research for broader synthesis.</p>
  </article>
  <article class="tour-surface-row">
    <p class="tour-surface-label">Saved items</p>
    <p><strong>Highlights and vocabulary.</strong> Save useful excerpts and define replacement rules for future summaries.</p>
  </article>
</div>

## Mobile Browse

The workspace opens in browse mode first. On mobile web, the compact rail keeps the main routes visible while the channel and recent-item list stay in the primary scroll area.

<figure class="tour-figure tour-figure--mobile">
  <img
    src="./images/ui-tour-mobile-browse.png"
    alt="Mobile workspace browse view showing the navigation rail, followed channels, recent items, and sync date context."
  />
  <figcaption>
    Mobile browse view. You can scan followed sources, recent items, and sync context before opening a specific item.
  </figcaption>
</figure>

- The navigation rail stays visible even on a phone-sized viewport.
- Browse stays focused on source selection and recent items instead of splitting attention with reading tools.
- Sync context stays visible in the browse flow so you can tell how far back the library reaches.

## Mobile Reading

Once you open an item, the layout shifts from browsing to reading. The tab strip keeps summary, transcript, highlights, and info in one place without hiding the surrounding route shell.

<figure class="tour-figure tour-figure--mobile">
  <img
    src="./images/ui-tour-mobile-workspace.png"
    alt="Mobile workspace reading view showing the summary tab, item actions, and the selected video's content."
  />
  <figcaption>
    Mobile reading view on the summary tab. The content is primary, while navigation and route changes stay lightweight around it.
  </figcaption>
</figure>

- Summary is the fastest way to triage an item on mobile.
- Info, transcript, and highlights stay one tap away in the same content strip.
- Item-level actions stay near the top of the reading view instead of moving into a separate menu.

## Queue

Queue is the operational route. It shows which items are still waiting on transcript or summary work and keeps the current channel's processing state in view.

<figure class="tour-figure tour-figure--mobile">
  <img
    src="./images/ui-tour-mobile-queue.png"
    alt="Mobile queue view showing actionable items, waiting items, failed work, and the processing status panel."
  />
  <figcaption>
    Mobile queue view. The same shell stays in place, but the main pane switches to processing status, waiting work, and failure context.
  </figcaption>
</figure>

- Queue is still scoped around the selected source.
- Processing counts are visible without leaving the page.
- The route stays useful even when the channel is nearly caught up because the state panel explains what the queue means.

## Chat

Chat uses the same library, but the workflow changes from reading one item to asking across many. On mobile web, the rail still gives you quick route changes while the conversation area stays focused on the draft and answer thread.

<figure class="tour-figure tour-figure--mobile">
  <img
    src="./images/ui-tour-mobile-chat.png"
    alt="Mobile chat view showing the route rail, starter prompts, and the message composer."
  />
  <figcaption>
    Mobile chat view. Starter prompts, the draft area, and deep-research controls stay visible without needing a separate settings step.
  </figcaption>
</figure>

- Anonymous chat stays available, but with limited quota and temporary history.
- Deep research and model choice stay in the conversation flow.
- Chat is strongest as a follow-up after you have already browsed or read a few items.

## Desktop Note

Desktop still uses the same routes and content states. The difference is density: desktop can show browsing and reading side by side, while mobile web keeps one primary task in focus.

## Additional Surfaces

Not every route needs a full screenshot to understand its role:

- `Channel overview` at `/channels/[id]` is the source-focused management view used when you want to inspect one channel without opening a selected video in the main reader.
- `Highlights` is the saved excerpt library built from transcript or summary selections.
- `Vocabulary` stores replacement rules for future summaries.
- `Login` supports guest browsing, standard web Google sign-in, and the Android system-browser auth handoff used by the Tauri shell.
- `Docs` is a separate VitePress frontend linked from the product header.
- `Guide` reopens the in-product walkthrough overlay from inside the workspace.

## Why This UI Shape Matters

The UI is built around content state, not just navigation. Transcript readiness, summary readiness, evaluation status, search coverage, and acknowledgement state all appear directly in the reading and queue flows. The backend sends the frontend enough state to keep those views current while the pipeline is still running.
