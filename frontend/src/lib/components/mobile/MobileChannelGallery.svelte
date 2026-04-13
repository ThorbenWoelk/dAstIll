<script lang="ts">
  import AddSourceDrawer from "$lib/components/AddSourceDrawer.svelte";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import type { Channel, ChannelSnapshot } from "$lib/types";
  import type { AddSourceSubmission } from "$lib/workspace/component-props";
  import { queueStageCardSummary } from "$lib/workspace/queue-stage-card-summary";

  let {
    channels,
    selectedChannelId,
    onSelectChannel,
    onAddChannel,
    addingChannel = false,
    loadingChannels = false,
    addSourceErrorMessage = null as string | null,
    /** When set with `queueUnifiedSummary`, cards show pipeline queue counts from each snapshot. */
    channelPreviews = undefined as Record<string, ChannelSnapshot> | undefined,
    queueUnifiedSummary = false,
  }: {
    channels: Channel[];
    selectedChannelId: string | null;
    onSelectChannel: (channelId: string) => void;
    /** When set, shows a + control and optional inline add form. */
    onAddChannel?: (input: AddSourceSubmission) => Promise<boolean> | boolean;
    addingChannel?: boolean;
    loadingChannels?: boolean;
    addSourceErrorMessage?: string | null;
    channelPreviews?: Record<string, ChannelSnapshot>;
    queueUnifiedSummary?: boolean;
  } = $props();

  let addDrawerOpen = $state(false);

  function toggleAddDrawer() {
    if (!onAddChannel) return;
    addDrawerOpen = !addDrawerOpen;
  }

  const normalizeThumbnail = (thumbnailUrl?: string | null): string | null => {
    const trimmed = thumbnailUrl?.trim();
    return trimmed ? trimmed : null;
  };

  const selectedIdx = $derived(
    selectedChannelId !== null
      ? channels.findIndex((c) => c.id === selectedChannelId)
      : -1,
  );
  const displayChannel = $derived(
    selectedIdx >= 0 ? channels[selectedIdx] : (channels[0] ?? null),
  );
  const displayIdx = $derived(selectedIdx >= 0 ? selectedIdx : 0);
  const prevChannelId = $derived(
    displayIdx > 0 ? (channels[displayIdx - 1]?.id ?? null) : null,
  );
  const nextChannelId = $derived(
    displayIdx >= 0 && displayIdx < channels.length - 1
      ? (channels[displayIdx + 1]?.id ?? null)
      : null,
  );

  // Preload adjacent thumbnails so swipe feels instant.
  $effect(() => {
    if (typeof window === "undefined") return;
    const urls = [prevChannelId, nextChannelId]
      .map((id) => {
        if (!id) return null;
        const ch = channels.find((c) => c.id === id);
        return ch ? normalizeThumbnail(ch.thumbnail_url) : null;
      })
      .filter((u): u is string => Boolean(u));
    for (const url of urls) {
      const img = new Image();
      img.referrerPolicy = "no-referrer";
      img.src = url;
    }
  });

  // Max dots shown before switching to "N / total" counter.
  const MAX_DOTS = 9;
</script>

<div class="lg:hidden">
  {#if loadingChannels && channels.length === 0}
    <!-- Loading skeleton -->
    <div
      class="relative h-[88px] w-full animate-pulse bg-[var(--muted)] opacity-60"
      aria-hidden="true"
    ></div>
    <div class="flex items-center gap-2 px-4 py-2">
      <div
        class="h-2 w-24 animate-pulse rounded-full bg-[var(--border)] opacity-50"
      ></div>
    </div>
  {:else if displayChannel}
    {@const thumb = normalizeThumbnail(displayChannel.thumbnail_url)}
    {@const preview = channelPreviews?.[displayChannel.id]}
    {@const queueLine =
      queueUnifiedSummary && preview
        ? queueStageCardSummary(preview.videos, "unified")
        : null}

    <!-- Channel banner -->
    <div class="relative h-[88px] w-full overflow-hidden bg-[var(--muted)]">
      <img
        src={thumb ?? defaultChannelIcon}
        alt={displayChannel.name}
        class="h-full w-full object-cover"
        fetchpriority="high"
        decoding="async"
        referrerpolicy="no-referrer"
      />
      <!-- Gradient for legibility -->
      <div
        class="absolute inset-0 bg-gradient-to-t from-black/65 via-black/15 to-transparent"
        aria-hidden="true"
      ></div>

      <!-- Channel name + subtitle -->
      <div class="absolute bottom-0 left-0 right-0 px-4 pb-2.5">
        <p class="truncate text-[14px] font-semibold leading-snug text-white">
          {displayChannel.name}
        </p>
        {#if queueLine}
          <p class="mt-0.5 truncate text-[11px] leading-tight text-white/70">
            {queueLine}
          </p>
        {:else if displayChannel.handle}
          <p class="mt-0.5 truncate text-[11px] leading-tight text-white/60">
            {displayChannel.handle}
          </p>
        {/if}
      </div>

      <!-- Prev arrow -->
      {#if prevChannelId}
        <button
          type="button"
          class="absolute left-2 top-1/2 flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-full bg-black/35 text-white transition-colors hover:bg-black/55 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/50"
          onclick={() => prevChannelId && onSelectChannel(prevChannelId)}
          aria-label="Previous channel"
        >
          <ChevronIcon direction="left" size={16} strokeWidth={2.5} />
        </button>
      {/if}

      <!-- Next arrow -->
      {#if nextChannelId}
        <button
          type="button"
          class="absolute right-2 top-1/2 flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-full bg-black/35 text-white transition-colors hover:bg-black/55 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/50"
          onclick={() => nextChannelId && onSelectChannel(nextChannelId)}
          aria-label="Next channel"
        >
          <ChevronIcon direction="right" size={16} strokeWidth={2.5} />
        </button>
      {/if}
    </div>

    <!-- Dot nav + add button -->
    {#if channels.length > 1 || onAddChannel}
      <div
        class="flex items-center gap-3 px-4 py-2"
        aria-label="Channel navigation"
      >
        <!-- Dots or counter -->
        <div class="flex min-w-0 flex-1 items-center gap-1.5">
          {#if channels.length <= MAX_DOTS}
            {#each channels as ch, i (ch.id)}
              <button
                type="button"
                class="h-1.5 shrink-0 rounded-full transition-all focus-visible:outline-none {i ===
                displayIdx
                  ? 'w-4 bg-[var(--accent)]'
                  : 'w-1.5 bg-[var(--soft-foreground)] opacity-25 hover:opacity-50'}"
                onclick={() => onSelectChannel(ch.id)}
                aria-label={ch.name}
                aria-current={i === displayIdx ? "true" : undefined}
              ></button>
            {/each}
          {:else}
            <span
              class="text-[11px] font-medium tabular-nums text-[var(--soft-foreground)] opacity-50"
            >
              {displayIdx + 1} / {channels.length}
            </span>
          {/if}
        </div>

        <!-- Add channel -->
        {#if onAddChannel}
          <button
            type="button"
            class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none {addDrawerOpen
              ? 'bg-[var(--accent-wash)] text-[var(--foreground)]'
              : ''}"
            onclick={toggleAddDrawer}
            aria-label={addDrawerOpen ? "Close add source" : "Add source"}
            aria-expanded={addDrawerOpen}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>
        {/if}
      </div>
    {/if}
  {/if}

  {#if onAddChannel}
    <AddSourceDrawer
      open={addDrawerOpen}
      busy={addingChannel}
      errorMessage={addSourceErrorMessage}
      onOpen={() => {
        addDrawerOpen = true;
      }}
      onClose={() => {
        addDrawerOpen = false;
      }}
      onSubmit={onAddChannel}
    />
  {/if}
</div>
