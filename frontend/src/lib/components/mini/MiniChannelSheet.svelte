<script lang="ts">
  import CloseIcon from "$lib/components/icons/CloseIcon.svelte";
  import type { Channel, MiniSummaryItem } from "$lib/transport-types";

  interface Props {
    open: boolean;
    channels: Channel[];
    selectedChannelId: string | null;
    summaries: MiniSummaryItem[];
    onSelect: (channelId: string) => void;
    onClose: () => void;
  }

  let {
    open,
    channels,
    selectedChannelId,
    summaries,
    onSelect,
    onClose,
  }: Props = $props();

  let brokenThumbs = $state(new Set<string>());

  function getChannelUnreadCount(channelId: string): number {
    return summaries.filter((s) => s.channel_id === channelId && !s.read)
      .length;
  }

  function handleThumbError(channelId: string) {
    brokenThumbs = new Set([...brokenThumbs, channelId]);
  }

  function handleSelect(channelId: string) {
    onSelect(channelId);
    onClose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

{#if open}
  <div
    class="sheet-backdrop"
    role="presentation"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
  >
    <div class="sheet" role="dialog" aria-label="Select channel">
      <div class="sheet-handle"></div>

      <div class="sheet-header">
        <span class="sheet-label">Channels</span>
        <button
          type="button"
          class="sheet-close"
          onclick={onClose}
          aria-label="Close"
        >
          <CloseIcon size={14} strokeWidth={2.5} />
        </button>
      </div>

      <div class="sheet-list">
        {#each channels as channel}
          {@const unread = getChannelUnreadCount(channel.id)}
          <button
            type="button"
            class="channel-row"
            class:channel-row--selected={channel.id === selectedChannelId}
            onclick={() => handleSelect(channel.id)}
          >
            <div class="channel-avatar">
              {#if channel.thumbnail_url && !brokenThumbs.has(channel.id)}
                <img
                  src={channel.thumbnail_url}
                  alt=""
                  class="channel-thumb"
                  onerror={() => handleThumbError(channel.id)}
                />
              {:else}
                <span class="channel-initial"
                  >{channel.name.charAt(0).toUpperCase()}</span
                >
              {/if}
            </div>
            <span class="channel-name">{channel.name}</span>
            {#if unread > 0}
              <span class="channel-unread">{unread}</span>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .sheet-backdrop {
    position: fixed;
    inset: 0;
    background: var(--surface-overlay-strong);
    z-index: calc(var(--z-mobile-tab-bar, 60) + 10);
    display: flex;
    align-items: flex-end;
    animation: backdrop-in 200ms ease;
  }
  @keyframes backdrop-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .sheet {
    width: 100%;
    max-height: 60dvh;
    background: var(--surface);
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    display: flex;
    flex-direction: column;
    animation: sheet-up 300ms cubic-bezier(0.32, 0.72, 0, 1);
  }
  @keyframes sheet-up {
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
  }

  .sheet-handle {
    width: 36px;
    height: 4px;
    border-radius: 2px;
    background: var(--border);
    margin: var(--space-sm) auto 0;
    flex-shrink: 0;
  }

  .sheet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-md);
    flex-shrink: 0;
  }
  .sheet-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--soft-foreground);
  }
  .sheet-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border-radius: var(--radius-full);
    border: none;
    background: transparent;
    color: var(--soft-foreground);
    cursor: pointer;
    transition: background 120ms;
  }
  .sheet-close:hover {
    background: var(--accent-wash);
    color: var(--foreground);
  }

  .sheet-list {
    overflow-y: auto;
    padding: 0 var(--space-sm) max(var(--space-md), env(safe-area-inset-bottom));
  }

  .channel-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    width: 100%;
    min-height: 52px;
    padding: var(--space-sm) var(--space-sm);
    border-radius: var(--radius-md);
    border: none;
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
    text-align: left;
    transition: background 120ms;
  }
  .channel-row:hover {
    background: var(--accent-wash);
  }
  .channel-row--selected {
    background: var(--accent-wash-strong);
  }

  .channel-avatar {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    overflow: hidden;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--muted);
    color: var(--soft-foreground);
    font-size: 13px;
    font-weight: 700;
  }
  .channel-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .channel-name {
    flex: 1;
    min-width: 0;
    font-size: 14px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .channel-unread {
    font-size: 10px;
    font-weight: 700;
    color: var(--accent);
    flex-shrink: 0;
  }
</style>
