<script lang="ts">
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import type { Channel, ChannelSnapshot, QueueTab } from "$lib/types";
  import { queueStageCardSummary } from "$lib/workspace/queue-stage-card-summary";

  import { tick } from "svelte";

  let {
    channels,
    selectedChannelId,
    onSelectChannel,
    onAddChannel,
    addingChannel = false,
    addSourceErrorMessage = null as string | null,
    /** When set with `queueTab`, cards show per-stage queue counts from each snapshot. */
    channelPreviews = undefined as Record<string, ChannelSnapshot> | undefined,
    queueTab = undefined as QueueTab | undefined,
  }: {
    channels: Channel[];
    selectedChannelId: string | null;
    onSelectChannel: (channelId: string) => void;
    /** When set, shows a + control and optional inline add form. */
    onAddChannel?: (input: string) => Promise<boolean> | boolean;
    addingChannel?: boolean;
    addSourceErrorMessage?: string | null;
    channelPreviews?: Record<string, ChannelSnapshot>;
    queueTab?: QueueTab;
  } = $props();

  let addFormOpen = $state(false);
  let addInput = $state("");
  let addInputEl = $state<HTMLInputElement | null>(null);

  async function toggleAddForm() {
    if (!onAddChannel) return;
    addFormOpen = !addFormOpen;
    if (!addFormOpen) {
      addInput = "";
      return;
    }
    await tick();
    addInputEl?.focus({ preventScroll: false });
  }

  async function handleAddSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (!onAddChannel) return;
    const submittedInput = addInput.trim();
    if (!submittedInput || addingChannel) return;
    addInput = "";
    const success = await onAddChannel(submittedInput);
    if (!success) {
      addInput = submittedInput;
      return;
    }
    addFormOpen = false;
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
</script>

<div class="lg:hidden">
  <div class="flex items-stretch gap-2 pl-4 pr-2 pt-3">
    <div
      bind:this={scrollerEl}
      class="custom-scrollbar flex min-w-0 max-w-full flex-1 flex-nowrap gap-3 overflow-x-auto pb-3 [-ms-overflow-style:none] [scrollbar-width:none] [scroll-padding-inline:1rem] [&::-webkit-scrollbar]:hidden"
      style="scroll-snap-type: x mandatory"
      aria-label="Channels"
    >
      {#each channels as channel (channel.id)}
        {@const thumb = normalizeThumbnail(channel.thumbnail_url)}
        {@const active = selectedChannelId === channel.id}
        {@const preview = channelPreviews?.[channel.id]}
        {@const queueLine =
          queueTab && preview
            ? queueStageCardSummary(preview.videos, queueTab)
            : null}
        <button
          use:registerCard={channel.id}
          type="button"
          class={`group relative snap-center flex w-[78vw] max-w-[20rem] shrink-0 flex-col overflow-hidden rounded-[var(--radius-lg)] bg-[var(--surface-strong)] shadow-sm transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${
            active
              ? "ring-1 ring-[var(--accent)]/25"
              : "hover:bg-[var(--panel-surface)]"
          }`}
          onclick={() => onSelectChannel(channel.id)}
          aria-current={active ? "true" : undefined}
          aria-label={channel.name}
        >
          <div class="relative h-28 w-full bg-[var(--muted)]">
            <img
              src={thumb ?? defaultChannelIcon}
              alt={channel.name}
              class="h-full w-full object-cover"
              loading="lazy"
              referrerpolicy="no-referrer"
            />
            <div
              class={`absolute inset-0 bg-gradient-to-t from-black/55 via-black/10 to-transparent transition-opacity ${
                active ? "opacity-100" : "opacity-80 group-hover:opacity-100"
              }`}
              aria-hidden="true"
            ></div>
          </div>
          <div class="flex min-w-0 flex-1 flex-col gap-1 px-4 py-3">
            <div class="min-w-0">
              <div
                class="truncate text-[15px] font-semibold text-[var(--foreground)]"
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
    </div>

    {#if onAddChannel}
      <div class="flex shrink-0 flex-col items-center justify-center pb-3">
        <button
          type="button"
          class="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 {addFormOpen
            ? 'bg-[var(--accent-wash)] text-[var(--foreground)]'
            : ''}"
          onclick={() => void toggleAddForm()}
          aria-label={addFormOpen ? "Close add channel" : "Add channel"}
          aria-expanded={addFormOpen}
        >
          <svg
            width="22"
            height="22"
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

  {#if onAddChannel && addFormOpen}
    <form
      class="px-4 pb-3"
      onsubmit={handleAddSubmit}
      aria-label="Add channel or video"
    >
      <div
        class="flex min-w-0 items-center gap-2 border-b border-[var(--accent-border-soft)] pb-1 transition-all focus-within:border-[var(--accent)]/40"
      >
        <label for="mobile-channel-add-input" class="sr-only"
          >Add channel or video</label
        >
        <input
          id="mobile-channel-add-input"
          bind:this={addInputEl}
          name="channel"
          autocomplete="off"
          spellcheck={false}
          class="min-w-0 flex-1 bg-transparent py-2 text-[14px] font-medium placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none"
          placeholder="Paste handle, channel URL, or video link"
          bind:value={addInput}
        />
        <button
          type="submit"
          class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-[var(--foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--accent-strong)] disabled:opacity-30"
          disabled={!addInput.trim() || addingChannel}
          aria-label="Submit add channel"
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
            ><line x1="12" y1="5" x2="12" y2="19" /><line
              x1="5"
              y1="12"
              x2="19"
              y2="12"
            /></svg
          >
        </button>
      </div>
      {#if addSourceErrorMessage}
        <p class="mt-2 text-[11px] font-medium text-[var(--danger)] opacity-90">
          {addSourceErrorMessage}
        </p>
      {/if}
    </form>
  {/if}
</div>
