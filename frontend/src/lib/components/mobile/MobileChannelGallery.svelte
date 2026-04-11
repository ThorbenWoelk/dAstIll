<script lang="ts">
  import AddSourceDrawer from "$lib/components/AddSourceDrawer.svelte";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import type { Channel, ChannelSnapshot } from "$lib/types";
  import type { AddSourceSubmission } from "$lib/workspace/component-props";
  import { queueStageCardSummary } from "$lib/workspace/queue-stage-card-summary";
  import { tick } from "svelte";

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

  let scrollerEl = $state<HTMLDivElement | null>(null);
  let cardEls = $state<Map<string, HTMLButtonElement>>(new Map());

  function setCardEl(channelId: string, el: HTMLButtonElement | null) {
    if (!el) {
      cardEls.delete(channelId);
      return;
    }
    cardEls.set(channelId, el);
  }

  function scrollSelectedCardToCenter() {
    const container = scrollerEl;
    if (!container) return;
    const selected = selectedChannelId;
    if (!selected) return;
    const el = cardEls.get(selected);
    if (!el) return;

    const elRect = el.getBoundingClientRect();
    const cr = container.getBoundingClientRect();
    const elCenterInContent =
      container.scrollLeft + (elRect.left - cr.left) + elRect.width / 2;
    const viewMid = container.clientWidth / 2;
    const maxScroll = Math.max(
      0,
      container.scrollWidth - container.clientWidth,
    );
    const nextLeft = Math.max(
      0,
      Math.min(elCenterInContent - viewMid, maxScroll),
    );

    container.scrollTo({ left: nextLeft, behavior: "smooth" });
  }

  $effect(() => {
    void channels;
    const selected = selectedChannelId;
    if (!selected) return;
    if (!scrollerEl) return;

    void tick().then(() => {
      scrollSelectedCardToCenter();
    });
  });

  function registerCard(node: HTMLButtonElement, channelId: string) {
    setCardEl(channelId, node);
    return {
      destroy() {
        setCardEl(channelId, null);
      },
    };
  }

  const EAGER_THUMB_COUNT = 12;

  /** Warm Cache API + browser cache for thumbnails past the eager strip (idle). */
  $effect(() => {
    if (typeof window === "undefined" || !("requestIdleCallback" in window)) {
      return;
    }
    const urls = channels
      .slice(EAGER_THUMB_COUNT)
      .map((c) => normalizeThumbnail(c.thumbnail_url))
      .filter((u): u is string => Boolean(u));
    if (urls.length === 0) return;

    const id = window.requestIdleCallback(
      () => {
        for (const url of urls.slice(0, 24)) {
          const img = new Image();
          img.referrerPolicy = "no-referrer";
          img.src = url;
        }
      },
      { timeout: 2000 },
    );
    return () => window.cancelIdleCallback(id);
  });
</script>

<div class="lg:hidden">
  <div class="pl-4 pr-2 pt-3">
    <div
      bind:this={scrollerEl}
      class="custom-scrollbar flex min-w-0 max-w-full flex-nowrap gap-2 overflow-x-auto pb-2 [-ms-overflow-style:none] [scrollbar-width:none] [scroll-padding-inline:1rem] [&::-webkit-scrollbar]:hidden"
      style="scroll-snap-type: x mandatory"
      aria-label="Sources"
    >
      {#if loadingChannels && channels.length === 0}
        {#each Array.from({ length: 4 }) as _, i (i)}
          <div
            class="flex w-[64vw] max-w-[14rem] shrink-0 snap-center flex-col overflow-hidden rounded-[var(--radius-md)] bg-[var(--surface-strong)] shadow-sm"
            aria-hidden="true"
          >
            <div
              class="h-20 w-full animate-pulse bg-[var(--border)] opacity-60"
            ></div>
            <div class="flex flex-col gap-1 px-3 py-2">
              <div
                class="h-3 w-3/4 animate-pulse rounded-full bg-[var(--border)] opacity-60"
              ></div>
              <div
                class="mt-1 h-2 w-1/2 animate-pulse rounded-full bg-[var(--border)] opacity-40"
              ></div>
            </div>
          </div>
        {/each}
      {/if}
      {#if loadingChannels && channels.length > 0}
        <!-- Subtle pulse dot during background channel refresh -->
        <div
          class="flex shrink-0 snap-center items-center justify-center self-stretch pl-1 pr-2"
          role="status"
          aria-live="polite"
          aria-label="Refreshing channels"
        >
          <span
            class="h-1.5 w-1.5 animate-pulse rounded-full bg-[var(--accent)] opacity-50"
            aria-hidden="true"
          ></span>
        </div>
      {/if}

      {#each channels as channel, index (channel.id)}
        {@const thumb = normalizeThumbnail(channel.thumbnail_url)}
        {@const active = selectedChannelId === channel.id}
        <!-- Lazy-loading hurts horizontal strips: off-axis images stay deferred. Eager first strip. -->
        {@const thumbLoading = index < EAGER_THUMB_COUNT ? "eager" : "lazy"}
        {@const thumbFetchPriority = active || index < 4 ? "high" : "auto"}
        {@const preview = channelPreviews?.[channel.id]}
        {@const queueLine =
          queueUnifiedSummary && preview
            ? queueStageCardSummary(preview.videos, "unified")
            : null}
        <button
          use:registerCard={channel.id}
          type="button"
          class={`group relative snap-center flex w-[64vw] max-w-[14rem] shrink-0 flex-col overflow-hidden rounded-[var(--radius-md)] bg-[var(--surface-strong)] shadow-sm transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${
            active
              ? "shadow-[0_14px_34px_color-mix(in_srgb,var(--foreground)_12%,transparent)]"
              : "hover:bg-[var(--panel-surface)]"
          }`}
          onclick={() => onSelectChannel(channel.id)}
          aria-current={active ? "true" : undefined}
          aria-label={channel.name}
        >
          <div class="relative h-20 w-full bg-[var(--muted)]">
            <img
              src={thumb ?? defaultChannelIcon}
              alt={channel.name}
              class="h-full w-full object-cover"
              loading={thumbLoading}
              decoding="async"
              fetchpriority={thumbFetchPriority}
              sizes="(max-width: 1024px) 64vw, 14rem"
              referrerpolicy="no-referrer"
            />
            <div
              class={`absolute inset-0 bg-gradient-to-t from-black/55 via-black/10 to-transparent transition-opacity ${
                active ? "opacity-100" : "opacity-80 group-hover:opacity-100"
              }`}
              aria-hidden="true"
            ></div>
          </div>
          <div class="flex min-w-0 flex-1 flex-col gap-1 px-3 py-2">
            <div class="min-w-0">
              <div
                class="truncate text-[13px] font-semibold leading-tight text-[var(--foreground)]"
              >
                {channel.name}
              </div>
              {#if queueLine}
                <div
                  class="mt-1 line-clamp-2 text-[11px] font-medium leading-snug text-[var(--soft-foreground)] opacity-80"
                >
                  {queueLine}
                </div>
              {:else}
                <div
                  class="mt-1 truncate text-[11px] font-medium text-[var(--soft-foreground)] opacity-60"
                >
                  {channel.handle ?? channel.id}
                </div>
              {/if}
            </div>
          </div>
        </button>
      {/each}

      {#if onAddChannel}
        <div
          class="flex w-9 shrink-0 snap-center flex-col items-center justify-center self-stretch pr-2"
          role="presentation"
        >
          <button
            type="button"
            class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 {addDrawerOpen
              ? 'bg-[var(--accent-wash)] text-[var(--foreground)]'
              : ''}"
            onclick={toggleAddDrawer}
            aria-label={addDrawerOpen
              ? "Close add source drawer"
              : "Add source"}
            aria-expanded={addDrawerOpen}
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>
        </div>
      {/if}
    </div>
  </div>

  {#if onAddChannel}
    <AddSourceDrawer
      open={addDrawerOpen}
      busy={addingChannel}
      errorMessage={addSourceErrorMessage}
      onClose={() => {
        addDrawerOpen = false;
      }}
      onSubmit={onAddChannel}
    />
  {/if}
</div>
