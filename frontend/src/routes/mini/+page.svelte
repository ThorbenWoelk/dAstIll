<script lang="ts">
  import { goto } from "$app/navigation";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte";
  import { authState } from "$lib/auth-state.svelte";
  import type { MiniReader } from "$lib/transport-types";
  import { getMiniReader, updateMiniReadStatus } from "$lib/api";
  import { renderMarkdown } from "$lib/utils/markdown";

  let reader = $state<MiniReader | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let selectedChannelId = $state<string | null>(null);
  let activeVideoId = $state<string | null>(null);
  let showUnreadOnly = $state(false);
  let markingRead = $state(false);
  let authResolved = $state(false);
  let scrollContainer = $state<HTMLElement | null>(null);
  let readProgress = $state(0);
  let contentKey = $state(0);

  let visibleSummaries = $derived(
    reader
      ? showUnreadOnly
        ? reader.summaries.filter((summary) => !summary.read)
        : reader.summaries
      : [],
  );
  let activeIndex = $derived(
    visibleSummaries.findIndex((summary) => summary.video_id === activeVideoId),
  );
  let activeSummary = $derived(
    activeIndex >= 0
      ? visibleSummaries[activeIndex]
      : (visibleSummaries[0] ?? null),
  );
  let activeSummaryHtml = $derived(
    activeSummary ? renderMarkdown(activeSummary.summary_content) : "",
  );

  let canGoPrev = $derived(activeIndex > 0);
  let canGoNext = $derived(
    activeIndex >= 0 && activeIndex < visibleSummaries.length - 1,
  );
  let unreadCount = $derived(
    reader?.summaries.filter((s) => !s.read).length ?? 0,
  );

  function formatDate(dateStr: string | null | undefined): string {
    if (!dateStr) return "";
    const d = new Date(dateStr);
    return d.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  function chooseActiveVideoId(
    summaries: MiniReader["summaries"],
    preferredVideoId?: string | null,
  ) {
    if (preferredVideoId) {
      const match = summaries.find(
        (summary) => summary.video_id === preferredVideoId,
      );
      if (match) return match.video_id;
    }

    const firstUnread = summaries.find((summary) => !summary.read);
    return firstUnread?.video_id ?? summaries[0]?.video_id ?? null;
  }

  async function loadReader(
    channelId?: string | null,
    preferredVideoId?: string | null,
  ) {
    loading = true;
    error = null;
    try {
      const nextReader = await getMiniReader(channelId);
      reader = nextReader;
      selectedChannelId = nextReader.selected_channel_id ?? null;
      activeVideoId = chooseActiveVideoId(
        nextReader.summaries,
        preferredVideoId,
      );
    } catch (cause) {
      reader = null;
      selectedChannelId = null;
      activeVideoId = null;
      error =
        cause instanceof Error ? cause.message : "Could not load dastill-mini.";
    } finally {
      loading = false;
    }
  }

  function stepSummary(delta: -1 | 1) {
    if (!activeSummary) return;
    const nextIndex = activeIndex + delta;
    const nextSummary = visibleSummaries[nextIndex];
    if (!nextSummary) return;
    activeVideoId = nextSummary.video_id;
    contentKey += 1;
    resetScroll();
  }

  function resetScroll() {
    if (scrollContainer) {
      scrollContainer.scrollTo({ top: 0, behavior: "instant" });
    }
    readProgress = 0;
  }

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    const maxScroll = scrollHeight - clientHeight;
    readProgress = maxScroll > 0 ? Math.min(1, scrollTop / maxScroll) : 0;
  }

  async function markActiveSummaryRead() {
    if (!activeSummary || markingRead) return;
    markingRead = true;
    error = null;
    try {
      await updateMiniReadStatus(activeSummary.video_id, true);
      if (!reader) return;
      reader = {
        ...reader,
        summaries: reader.summaries.map((summary) =>
          summary.video_id === activeSummary.video_id
            ? { ...summary, read: true }
            : summary,
        ),
      };
      const nextVisibleSummaries = showUnreadOnly
        ? reader.summaries.filter(
            (summary) =>
              !(summary.video_id === activeSummary.video_id) && !summary.read,
          )
        : reader.summaries.map((summary) =>
            summary.video_id === activeSummary.video_id
              ? { ...summary, read: true }
              : summary,
          );
      activeVideoId = chooseActiveVideoId(
        nextVisibleSummaries,
        activeSummary.video_id,
      );
      contentKey += 1;
      resetScroll();
    } catch (cause) {
      error =
        cause instanceof Error
          ? cause.message
          : "Could not update read status.";
    } finally {
      markingRead = false;
    }
  }

  async function handleChannelChange(event: Event) {
    const nextChannelId = (event.currentTarget as HTMLSelectElement).value;
    if (!nextChannelId || nextChannelId === selectedChannelId) return;
    selectedChannelId = nextChannelId;
    await loadReader(nextChannelId);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.target instanceof HTMLSelectElement) return;

    switch (event.key) {
      case "ArrowLeft":
      case "j":
        event.preventDefault();
        stepSummary(-1);
        break;
      case "ArrowRight":
      case "k":
        event.preventDefault();
        stepSummary(1);
        break;
      case "r":
        if (!activeSummary?.read) {
          event.preventDefault();
          void markActiveSummaryRead();
        }
        break;
    }
  }

  function jumpToSummary(videoId: string) {
    activeVideoId = videoId;
    contentKey += 1;
    resetScroll();
  }

  function toggleUnreadFilter() {
    showUnreadOnly = !showUnreadOnly;
  }

  $effect(() => {
    if (!reader) return;
    const nextActiveVideoId = chooseActiveVideoId(
      visibleSummaries,
      activeVideoId,
    );
    if (nextActiveVideoId !== activeVideoId) {
      activeVideoId = nextActiveVideoId;
      contentKey += 1;
      resetScroll();
    }
  });

  $effect(() => {
    if (!authState.ready || authResolved) return;
    authResolved = true;

    if (authState.current.authState !== "authenticated") {
      void goto("/login?redirectTo=%2Fmini");
      return;
    }

    void loadReader();
  });
</script>

<svelte:head>
  <title>dastill-mini</title>
  <meta
    name="description"
    content="A minimal summary reader for your subscribed dAstIll channels."
  />
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="mini-shell">
  <!-- Reading progress bar -->
  {#if activeSummary}
    <div class="progress-track">
      <div
        class="progress-fill"
        style="transform: scaleX({readProgress})"
      ></div>
    </div>
  {/if}

  <!-- Sticky top bar -->
  <header class="mini-bar">
    <div class="bar-left">
      <a class="bar-logo" href="/">dastill</a>
      <span class="bar-sep"></span>
      <span class="bar-label">mini</span>
    </div>

    {#if reader && reader.channels.length > 0}
      <div class="bar-center">
        <select
          class="channel-select"
          disabled={loading}
          value={selectedChannelId ?? ""}
          onchange={handleChannelChange}
        >
          {#each reader.channels as channel}
            <option value={channel.id}>{channel.name}</option>
          {/each}
        </select>
        {#if unreadCount > 0}
          <span class="unread-badge">{unreadCount} unread</span>
        {/if}
        <button
          type="button"
          class="filter-chip"
          class:filter-chip--active={showUnreadOnly}
          onclick={toggleUnreadFilter}
        >
          {showUnreadOnly ? "Showing unread" : "Hide read"}
        </button>
      </div>
    {/if}

    <div class="bar-right">
      {#if activeSummary && reader}
        <div class="bar-nav">
          <button
            type="button"
            class="nav-btn"
            disabled={!canGoPrev}
            onclick={() => stepSummary(-1)}
            aria-label="Previous summary"
          >
            <ChevronIcon direction="left" size={14} strokeWidth={2.4} />
          </button>
          <span class="nav-pos"
            >{activeIndex + 1}<span class="nav-pos-sep">/</span
            >{visibleSummaries.length}</span
          >
          <button
            type="button"
            class="nav-btn"
            disabled={!canGoNext}
            onclick={() => stepSummary(1)}
            aria-label="Next summary"
          >
            <ChevronIcon direction="right" size={14} strokeWidth={2.4} />
          </button>
        </div>
      {/if}
    </div>
  </header>

  <!-- Main content area -->
  <div class="mini-content" bind:this={scrollContainer} onscroll={handleScroll}>
    {#if loading}
      <div class="empty-state">
        <div class="empty-inner">
          <div class="loading-pulse"></div>
          <p class="empty-body">Loading reader</p>
        </div>
      </div>
    {:else if error}
      <div class="empty-state">
        <div class="empty-inner">
          <p class="empty-label">Reader unavailable</p>
          <p class="empty-body">{error}</p>
        </div>
      </div>
    {:else if !reader || reader.channels.length === 0}
      <div class="empty-state">
        <div class="empty-inner">
          <p class="empty-label">No subscriptions</p>
          <h2 class="empty-title">Nothing to read yet</h2>
          <p class="empty-body">
            dastill-mini shows summaries from channels you subscribed to. Add
            subscriptions in the full app when the main workspace is back.
          </p>
        </div>
      </div>
    {:else if !activeSummary}
      <div class="empty-state">
        <div class="empty-inner">
          {#if showUnreadOnly && reader?.summaries.length}
            <p class="empty-label">No unread summaries</p>
            <h2 class="empty-title">You’re all caught up</h2>
            <p class="empty-body">
              Every long-form summary in this channel is already marked as read.
            </p>
          {:else}
            <p class="empty-label">No summaries</p>
            <h2 class="empty-title">This channel is quiet for now</h2>
            <p class="empty-body">
              The selected channel has no long-form readable videos available
              yet.
            </p>
          {/if}
        </div>
      </div>
    {:else}
      <!-- Summary strip -->
      {#if visibleSummaries.length > 1}
        <nav class="strip" aria-label="Summary list">
          {#each visibleSummaries as summary, i}
            <button
              type="button"
              class="strip-item"
              class:strip-item--active={summary.video_id === activeVideoId}
              class:strip-item--read={summary.read}
              onclick={() => jumpToSummary(summary.video_id)}
            >
              {#if summary.thumbnail_url}
                <img
                  class="strip-thumb"
                  src={summary.thumbnail_url}
                  alt=""
                  loading="lazy"
                />
              {:else}
                <div class="strip-thumb strip-thumb--empty">
                  <span class="strip-thumb-num">{i + 1}</span>
                </div>
              {/if}
              <span class="strip-title">{summary.title}</span>
              {#if summary.read}
                <span class="strip-read-dot"><CheckIcon size={10} /></span>
              {/if}
            </button>
          {/each}
        </nav>
      {/if}

      <!-- Article -->
      {#key contentKey}
        <article class="reader-article">
          <!-- Hero area -->
          {#if activeSummary.thumbnail_url}
            <div class="hero-thumb-wrap">
              <img
                class="hero-thumb"
                src={activeSummary.thumbnail_url}
                alt={activeSummary.title}
              />
            </div>
          {/if}

          <div class="reader-header">
            <div class="reader-meta">
              <span class="reader-channel">{activeSummary.channel_name}</span>
              {#if activeSummary.published_at}
                <span class="reader-date"
                  >{formatDate(activeSummary.published_at)}</span
                >
              {/if}
            </div>

            <h1 class="reader-title">{activeSummary.title}</h1>

            <div class="reader-actions">
              {#if activeSummary.read}
                <span class="read-pill read-pill--done">
                  <CheckIcon size={12} className="read-pill-icon" />
                  Read
                </span>
              {:else}
                <button
                  type="button"
                  class="read-pill read-pill--action"
                  disabled={markingRead}
                  onclick={markActiveSummaryRead}
                >
                  {#if markingRead}
                    Saving
                  {:else}
                    Mark as read
                    <span class="shortcut-hint">R</span>
                  {/if}
                </button>
              {/if}

              <a
                class="source-link"
                href={activeSummary.watch_url}
                rel="noopener noreferrer"
                target="_blank"
              >
                Watch source
                <ExternalLinkIcon size={12} />
              </a>
            </div>
          </div>

          <div class="reader-body" aria-live="polite">
            {@html activeSummaryHtml}
          </div>

          <!-- Bottom nav for long articles -->
          <footer class="reader-footer">
            <div class="footer-nav">
              {#if canGoPrev}
                <button
                  type="button"
                  class="footer-btn"
                  onclick={() => stepSummary(-1)}
                >
                  <ChevronIcon direction="left" size={12} strokeWidth={2.4} />
                  Previous
                </button>
              {:else}
                <span></span>
              {/if}

              {#if canGoNext}
                <button
                  type="button"
                  class="footer-btn"
                  onclick={() => stepSummary(1)}
                >
                  Next
                  <ChevronIcon direction="right" size={12} strokeWidth={2.4} />
                </button>
              {:else}
                <span></span>
              {/if}
            </div>

            <p class="footer-keys">
              Use arrow keys to navigate, R to mark read
            </p>
          </footer>
        </article>
      {/key}
    {/if}
  </div>
</div>

<style>
  /* ── Shell ── */
  .mini-shell {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    background: var(--background);
    color: var(--foreground);
    overflow: hidden;
    position: relative;
  }

  /* ── Progress ── */
  .progress-track {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    z-index: 10;
    background: transparent;
  }
  .progress-fill {
    height: 100%;
    background: var(--accent);
    transform-origin: left;
    transition: transform 80ms linear;
    opacity: 0.7;
  }

  /* ── Top bar ── */
  .mini-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-lg);
    border-bottom: 1px solid var(--border-soft);
    flex-shrink: 0;
    min-height: 48px;
  }
  .bar-left {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
  }
  .bar-logo {
    font-family: "Fraunces", serif;
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--foreground);
    text-decoration: none;
  }
  .bar-logo:hover {
    color: var(--accent);
  }
  .bar-sep {
    width: 1px;
    height: 14px;
    background: var(--border);
  }
  .bar-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--soft-foreground);
  }
  .bar-center {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex: 1;
    justify-content: center;
    min-width: 0;
  }
  .channel-select {
    appearance: none;
    background: var(--surface);
    border: 1px solid var(--border-soft);
    border-radius: var(--radius-full);
    padding: 6px 16px;
    font-size: 12px;
    font-weight: 600;
    color: var(--foreground);
    cursor: pointer;
    outline: none;
    max-width: 240px;
    text-overflow: ellipsis;
    transition:
      border-color 150ms,
      background 150ms;
  }
  .channel-select:focus {
    border-color: var(--foreground);
  }
  .channel-select:hover {
    background: var(--accent-wash);
  }
  .unread-badge {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--accent);
    flex-shrink: 0;
  }
  .filter-chip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-soft);
    background: var(--surface);
    color: var(--foreground);
    padding: 6px 12px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    cursor: pointer;
    transition:
      background 120ms,
      border-color 120ms,
      color 120ms;
    flex-shrink: 0;
  }
  .filter-chip:hover {
    background: var(--accent-wash);
  }
  .filter-chip--active {
    background: var(--accent-wash-strong);
    border-color: var(--accent);
    color: var(--accent);
  }
  .bar-right {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
  }
  .bar-nav {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .nav-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    background: transparent;
    border: none;
    color: var(--foreground);
    cursor: pointer;
    transition: background 120ms;
  }
  .nav-btn:hover:not(:disabled) {
    background: var(--accent-wash);
  }
  .nav-btn:disabled {
    opacity: 0.25;
    cursor: default;
  }
  .nav-pos {
    font-size: 12px;
    font-weight: 600;
    color: var(--foreground);
    min-width: 36px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .nav-pos-sep {
    color: var(--soft-foreground);
    margin: 0 1px;
  }

  /* ── Content scroll ── */
  .mini-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior-y: contain;
    scroll-behavior: smooth;
  }

  /* ── Empty states ── */
  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 60dvh;
    padding: var(--space-xl);
  }
  .empty-inner {
    max-width: 400px;
    text-align: center;
  }
  .empty-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--soft-foreground);
  }
  .empty-title {
    font-family: "Fraunces", serif;
    font-size: 28px;
    letter-spacing: -0.03em;
    margin-top: var(--space-sm);
  }
  .empty-body {
    font-size: 14px;
    line-height: 1.7;
    color: var(--soft-foreground);
    margin-top: var(--space-md);
  }
  .loading-pulse {
    width: 32px;
    height: 3px;
    border-radius: 2px;
    background: var(--accent);
    margin: 0 auto var(--space-md);
    animation: pulse-width 1.4s ease-in-out infinite;
  }
  @keyframes pulse-width {
    0%,
    100% {
      transform: scaleX(0.4);
      opacity: 0.4;
    }
    50% {
      transform: scaleX(1);
      opacity: 1;
    }
  }

  /* ── Summary strip ── */
  .strip {
    display: flex;
    gap: var(--space-sm);
    padding: var(--space-md) var(--space-lg);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    border-bottom: 1px solid var(--border-soft);
    flex-shrink: 0;
  }
  .strip::-webkit-scrollbar {
    display: none;
  }
  .strip-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 6px 12px 6px 6px;
    border-radius: var(--radius-full);
    border: none;
    background: var(--surface);
    color: var(--foreground);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
    transition:
      background 120ms,
      box-shadow 120ms;
    flex-shrink: 0;
  }
  .strip-item:hover {
    background: var(--accent-wash);
  }
  .strip-item--active {
    background: var(--accent-wash-strong);
    font-weight: 700;
  }
  .strip-item--read {
    color: var(--soft-foreground);
  }
  .strip-thumb {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    object-fit: cover;
    flex-shrink: 0;
  }
  .strip-thumb--empty {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--muted);
    color: var(--soft-foreground);
  }
  .strip-thumb-num {
    font-size: 10px;
    font-weight: 700;
  }
  .strip-title {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .strip-read-dot {
    color: var(--soft-foreground);
    flex-shrink: 0;
  }

  /* ── Article ── */
  .reader-article {
    max-width: 680px;
    margin: 0 auto;
    padding: var(--space-xl) var(--space-lg) 80px;
    animation: article-in 400ms ease-out;
  }
  @keyframes article-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .hero-thumb-wrap {
    margin-bottom: var(--space-lg);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .hero-thumb {
    display: block;
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
  }

  .reader-header {
    margin-bottom: var(--space-xl);
  }
  .reader-meta {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }
  .reader-channel {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .reader-date {
    font-size: 12px;
    color: var(--soft-foreground);
  }
  .reader-title {
    font-family: "Fraunces", serif;
    font-size: 32px;
    font-weight: 600;
    letter-spacing: -0.03em;
    line-height: 1.15;
    font-variation-settings: "opsz" 72;
    margin-bottom: var(--space-md);
  }
  @media (min-width: 640px) {
    .reader-title {
      font-size: 40px;
    }
  }
  .reader-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-wrap: wrap;
  }
  .read-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 14px;
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    border: none;
    cursor: default;
    transition:
      background 120ms,
      color 120ms;
  }
  .read-pill--done {
    background: var(--surface);
    color: var(--soft-foreground);
  }
  :global(.read-pill-icon) {
    flex-shrink: 0;
  }
  .read-pill--action {
    background: var(--foreground);
    color: var(--background);
    cursor: pointer;
  }
  .read-pill--action:hover:not(:disabled) {
    background: var(--accent);
  }
  .read-pill--action:disabled {
    opacity: 0.6;
    cursor: wait;
  }
  .shortcut-hint {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--background) 20%, transparent);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0;
    margin-left: 4px;
  }
  .source-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 14px;
    border-radius: var(--radius-full);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--foreground);
    background: var(--surface);
    text-decoration: none;
    transition: background 120ms;
  }
  .source-link:hover {
    background: var(--accent-wash);
  }

  /* ── Reader body (markdown) ── */
  .reader-body {
    color: var(--foreground);
    font-size: 1rem;
    line-height: 1.9;
  }
  .reader-body :global(h1),
  .reader-body :global(h2),
  .reader-body :global(h3) {
    font-family: "Fraunces", serif;
    letter-spacing: -0.02em;
    line-height: 1.2;
    margin-top: 2rem;
    margin-bottom: 0.75rem;
  }
  .reader-body :global(h2) {
    font-size: 1.4rem;
  }
  .reader-body :global(h3) {
    font-size: 1.15rem;
  }
  .reader-body :global(p) {
    margin: 0;
  }
  .reader-body :global(p + p) {
    margin-top: 1rem;
  }
  .reader-body :global(ul),
  .reader-body :global(ol) {
    margin: 1rem 0;
    padding-left: 1.25rem;
  }
  .reader-body :global(li + li) {
    margin-top: 0.35rem;
  }
  .reader-body :global(a) {
    color: var(--foreground);
    text-decoration: underline;
    text-decoration-color: var(--accent);
    text-underline-offset: 0.22em;
  }
  .reader-body :global(a:hover) {
    color: var(--accent);
  }
  .reader-body :global(strong) {
    color: var(--foreground);
    font-weight: 700;
  }
  .reader-body :global(blockquote) {
    border-left: 2px solid var(--border);
    padding-left: var(--space-md);
    color: var(--soft-foreground);
    margin: 1.5rem 0;
  }
  .reader-body :global(code) {
    font-size: 0.9em;
    background: var(--muted);
    padding: 2px 6px;
    border-radius: 4px;
  }
  .reader-body :global(pre) {
    background: var(--surface);
    padding: var(--space-md);
    border-radius: var(--radius-sm);
    overflow-x: auto;
    margin: 1.5rem 0;
  }
  .reader-body :global(pre code) {
    background: none;
    padding: 0;
  }

  /* ── Footer nav ── */
  .reader-footer {
    margin-top: 64px;
    padding-top: var(--space-lg);
    border-top: 1px solid var(--border-soft);
    text-align: center;
  }
  .footer-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-md);
  }
  .footer-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border-radius: var(--radius-full);
    border: none;
    background: var(--surface);
    color: var(--foreground);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background 120ms;
  }
  .footer-btn:hover {
    background: var(--accent-wash);
  }
  .footer-keys {
    font-size: 11px;
    color: var(--soft-foreground);
    letter-spacing: 0.02em;
  }

  /* ── Mobile adjustments ── */
  @media (max-width: 640px) {
    .mini-bar {
      padding: var(--space-sm) var(--space-md);
      gap: var(--space-sm);
    }
    .bar-center {
      display: none;
    }
    .reader-article {
      padding: var(--space-lg) var(--space-md) 80px;
    }
    .reader-title {
      font-size: 26px;
    }
    .strip {
      padding: var(--space-sm) var(--space-md);
    }
  }
</style>
