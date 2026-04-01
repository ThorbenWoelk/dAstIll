---
title: UI Tour
pageClass: ui-tour-page
---

# UI Tour

<div class="tour-hero">
  <div>
    <p class="tour-eyebrow">Visual walkthrough</p>
    <p class="tour-lede">
      dAstIll is organized around one loop: monitor channels, open a video, read the distilled version,
      then branch into queue or chat when you need operational visibility or cross-video answers.
    </p>
    <div class="tour-route-strip">
      <span>Workspace</span>
      <span>Queue</span>
      <span>Highlights</span>
      <span>Vocabulary</span>
      <span>Chat</span>
      <span>Docs</span>
    </div>
  </div>
  <div class="tour-meta-grid">
    <article class="tour-meta-card">
      <p class="tour-meta-label">Capture Date</p>
      <strong>April 1, 2026</strong>
      <p>All screenshots below were recaptured from the deployed app for this refresh.</p>
    </article>
    <article class="tour-meta-card">
      <p class="tour-meta-label">Primary Loop</p>
      <strong>Browse, read, triage, ask</strong>
      <p>The routes are distinct, but they share one content model and one section shell.</p>
    </article>
    <article class="tour-meta-card">
      <p class="tour-meta-label">Signed-Out Experience</p>
      <strong>Still useful</strong>
      <p>Anonymous browsing and quota-limited chat remain available even before sign-in.</p>
    </article>
    <article class="tour-meta-card">
      <p class="tour-meta-label">Guide Entry Point</p>
      <strong>In product</strong>
      <p>The header-level Guide control opens the built-in walkthrough from the workspace.</p>
    </article>
  </div>
</div>

## Surface Map

<div class="tour-surface-grid">
  <article class="tour-surface-card">
    <p class="tour-surface-label">Workspace</p>
    <h3>Default operating surface</h3>
    <p>Browse channels, scan recent uploads, and switch the selected video between info, summary, highlights, and transcript.</p>
  </article>
  <article class="tour-surface-card">
    <p class="tour-surface-label">Queue</p>
    <h3>Operational backlog</h3>
    <p>Track transcript extraction, summary generation, failures, and backfill depth without leaving the main shell.</p>
  </article>
  <article class="tour-surface-card">
    <p class="tour-surface-label">Chat</p>
    <h3>Grounded assistant</h3>
    <p>Run RAG conversations over the same library with streamed tool steps, evidence, and source-aware replies.</p>
  </article>
  <article class="tour-surface-card">
    <p class="tour-surface-label">Personal library</p>
    <h3>Highlights and vocabulary</h3>
    <p>Save excerpts worth keeping and define replacement rules that future summaries should normalize toward.</p>
  </article>
</div>

## Workspace

The workspace is the product's center of gravity. It keeps global navigation on the far left, channel and video browsing in the middle, and the selected content view on the right. On desktop that means you can move from channel discovery to deep reading without losing orientation.

<figure class="tour-figure tour-figure--wide">
  <img
    src="./images/ui-tour-workspace-desktop.png"
    alt="Desktop workspace showing the section rail, channel list, recent videos, and the selected video's AI summary."
  />
  <figcaption>
    Desktop workspace. The selected video stays in focus while the main pane swaps between info, summary, highlights, and transcript.
  </figcaption>
</figure>

- The left rail anchors route changes without collapsing the rest of the reading context.
- The middle column behaves like a channel inbox: recent uploads first, browsing second.
- The content pane is optimized for reading, with summary and transcript treated as first-class views rather than secondary drawers.
- Signed-out visitors still land in a usable workspace via the seeded default channel.

## Queue

The queue is not a second library view. It is the operational readout for content still moving through the pipeline. When work is pending, this route surfaces incomplete transcripts, missing summaries, retries, and sync boundaries. When the system is caught up, the page becomes a "clear" state rather than disappearing.

<figure class="tour-figure tour-figure--wide">
  <img
    src="./images/ui-tour-queue-desktop.png"
    alt="Desktop queue view showing a clear processing queue state, sync depth controls, and the shared section shell."
  />
  <figcaption>
    Queue route in a clear state on April 1, 2026. The layout stays useful even when no items are currently waiting.
  </figcaption>
</figure>

- Queue state is still channel-scoped, so operators can inspect one feed at a time.
- Sync depth lives here because backfill policy is an operational control, not a reading preference.
- The route reuses the same shell and sidebar language as the workspace, which keeps the mode switch lightweight.

## Chat

Chat sits on top of the same transcript and summary corpus, but the interaction model is different: instead of selecting one video and reading linearly, you ask for synthesis, explanation, or retrieval across the library. The composer keeps model choice and deep-research mode in the conversation workflow rather than hiding them in settings.

<figure class="tour-figure tour-figure--wide">
  <img
    src="./images/ui-tour-chat-desktop.png"
    alt="Desktop chat view showing an anonymous prompt about leaked source code, streamed planning steps, library search activity, and a grounded answer."
  />
  <figcaption>
    Chat during a live anonymous prompt. The UI streams tool progress, shows invoked search work, and then renders a grounded answer in the same thread.
  </figcaption>
</figure>

- Conversations are first-class objects with their own sidebar, rename flow, and delete controls.
- Anonymous usage is allowed, but it is intentionally quota-limited and conversation history is ephemeral.
- Tool progress is visible instead of hidden, which makes the assistant feel like part of the product runtime instead of a black box.

## Mobile

On mobile the shell collapses around the reading experience. The tab strip stays at the top of the content stack, while bottom navigation handles route changes. The result is a tighter, more article-like presentation that still preserves the product's major modes.

<figure class="tour-figure tour-figure--mobile">
  <img
    src="./images/ui-tour-mobile-workspace.png"
    alt="Mobile workspace showing the selected video's summary view, tab strip, and bottom navigation."
  />
  <figcaption>
    Mobile workspace summary view. Reading remains primary, while section switches move to the bottom navigation bar.
  </figcaption>
</figure>

- The mobile layout prioritizes the selected video's content over the surrounding library chrome.
- The same content modes remain available, but they are surfaced as a compact top tab strip.
- Navigation, queue, highlights, and chat stay reachable without forcing a separate mobile-only information architecture.

## Additional Surfaces

Not every route needs a full screenshot to understand its role:

- `Highlights` is the saved excerpt library built from transcript or summary selections.
- `Vocabulary` stores replacement rules that future summaries should normalize toward.
- `Docs` is a separate VitePress frontend linked from the product header.
- `Guide` reopens the in-product walkthrough overlay when users need a refresher inside the workspace itself.

## Why This UI Shape Matters

The UI is built around backend lifecycle state, not just navigation. Transcript readiness, summary readiness, evaluation status, search coverage, and acknowledgement state all surface directly in the reading and queue flows. That is why the backend exposes rich bootstrap payloads and why the frontend keeps route transitions lightweight: the product is designed to stay readable while the content pipeline keeps changing underneath it.
