<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";

  interface Props {
    channelName: string | null;
    canGoPrev: boolean;
    canGoNext: boolean;
    activeIndex: number;
    totalCount: number;
    showReadCheckbox: boolean;
    activeSummaryRead: boolean;
    markingRead: boolean;
    onPrev: () => void;
    onNext: () => void;
    onOpenChannelPicker: () => void;
    onMarkReadAndAdvance: () => void | Promise<void>;
  }

  let {
    channelName,
    canGoPrev,
    canGoNext,
    activeIndex,
    totalCount,
    showReadCheckbox,
    activeSummaryRead,
    markingRead,
    onPrev,
    onNext,
    onOpenChannelPicker,
    onMarkReadAndAdvance,
  }: Props = $props();

  async function handleReadToggle() {
    if (activeSummaryRead || markingRead) return;
    await onMarkReadAndAdvance();
  }
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

  {#if showReadCheckbox}
    <button
      type="button"
      class="read-stamp"
      class:read-stamp--done={activeSummaryRead}
      class:read-stamp--loading={markingRead}
      disabled={activeSummaryRead || markingRead}
      onclick={handleReadToggle}
      aria-label={activeSummaryRead ? "Already read" : "Mark read and advance"}
      aria-pressed={activeSummaryRead}
      data-tooltip={activeSummaryRead ? "Read" : "Mark read"}
      data-tooltip-placement="top"
    >
      <span class="read-stamp-ring" aria-hidden="true">
        <CheckIcon size={14} strokeWidth={2.4} className="read-stamp-check" />
      </span>
    </button>
  {:else}
    <button type="button" class="channel-trigger" onclick={onOpenChannelPicker}>
      <span class="channel-name">{channelName ?? "Select channel"}</span>
      <ChevronIcon direction="down" size={10} strokeWidth={2.4} />
    </button>
  {/if}

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
    background: var(--background);
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
    transition:
      background 120ms,
      color 120ms,
      box-shadow 120ms;
  }
  .nav-btn:active:not(:disabled) {
    background: var(--accent-wash);
  }
  .nav-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 40%, transparent);
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
  .channel-name {
    font-size: 13px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .read-stamp {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
    flex-shrink: 0;
  }
  .read-stamp-ring {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    border: 1.5px solid var(--foreground);
    background: transparent;
    color: transparent;
    transition:
      background 180ms ease,
      color 180ms ease,
      transform 180ms ease,
      border-color 180ms ease;
  }
  :global(.read-stamp-check) {
    transform: scale(0.7);
    opacity: 0;
    transition:
      transform 180ms cubic-bezier(0.32, 0.72, 0, 1),
      opacity 140ms ease;
  }
  .read-stamp:active:not(:disabled) .read-stamp-ring {
    transform: scale(0.92);
  }
  .read-stamp--done .read-stamp-ring,
  .read-stamp--loading .read-stamp-ring {
    background: var(--foreground);
    color: var(--background);
  }
  .read-stamp--done :global(.read-stamp-check),
  .read-stamp--loading :global(.read-stamp-check) {
    transform: scale(1);
    opacity: 1;
  }
  .read-stamp--loading .read-stamp-ring {
    animation: stamp-pulse 1s ease-in-out infinite;
  }
  .read-stamp:disabled {
    cursor: default;
  }
  @keyframes stamp-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.55;
    }
  }

  @media (hover: hover) and (pointer: fine) {
    .nav-btn:hover:not(:disabled) {
      color: var(--accent-strong);
    }
    .channel-trigger:hover {
      background: var(--accent-wash);
    }
    .read-stamp:hover:not(:disabled) .read-stamp-ring {
      background: var(--foreground);
      color: var(--background);
    }
    .read-stamp:hover:not(:disabled) :global(.read-stamp-check) {
      transform: scale(1);
      opacity: 1;
    }
  }

  @media (min-width: 960px) {
    .bottom-bar {
      display: none;
    }
  }
</style>
