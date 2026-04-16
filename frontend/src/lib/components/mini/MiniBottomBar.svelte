<script lang="ts">
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";

  interface Props {
    channelName: string | null;
    canGoPrev: boolean;
    canGoNext: boolean;
    activeIndex: number;
    totalCount: number;
    onPrev: () => void;
    onNext: () => void;
    onOpenChannelPicker: () => void;
  }

  let {
    channelName,
    canGoPrev,
    canGoNext,
    activeIndex,
    totalCount,
    onPrev,
    onNext,
    onOpenChannelPicker,
  }: Props = $props();
</script>

<nav class="bottom-bar" aria-label="Reader navigation">
  <button
    type="button"
    class="nav-btn"
    disabled={!canGoPrev}
    onclick={onPrev}
    aria-label="Previous summary"
  >
    <ChevronIcon direction="left" size={18} strokeWidth={2.2} />
  </button>

  <button type="button" class="channel-trigger" onclick={onOpenChannelPicker}>
    <span class="channel-name">{channelName ?? "Select channel"}</span>
    <ChevronIcon direction="down" size={10} strokeWidth={2.4} />
  </button>

  <button
    type="button"
    class="nav-btn"
    disabled={!canGoNext}
    onclick={onNext}
    aria-label="Next summary"
  >
    <ChevronIcon direction="right" size={18} strokeWidth={2.2} />
  </button>
</nav>

<style>
  .bottom-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    padding-bottom: max(var(--space-sm), env(safe-area-inset-bottom));
    background: var(--surface);
    border-top: 1px solid var(--border-soft);
    z-index: var(--z-mobile-tab-bar, 60);
    min-height: 52px;
  }

  .nav-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    border-radius: var(--radius-full);
    border: none;
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
    flex-shrink: 0;
    transition: background 120ms;
  }
  .nav-btn:hover:not(:disabled) {
    background: var(--accent-wash);
  }
  .nav-btn:disabled {
    opacity: 0.25;
    cursor: default;
    pointer-events: none;
  }

  .channel-trigger {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-md);
    border-radius: var(--radius-full);
    border: none;
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
    transition: background 120ms;
    min-height: 44px;
    max-width: 280px;
  }
  .channel-trigger:hover {
    background: var(--accent-wash);
  }
  .channel-name {
    font-size: 13px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
