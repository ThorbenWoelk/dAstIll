<script lang="ts">
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

  async function handleReadCheckboxChange(event: Event) {
    const input = event.currentTarget;
    if (!(input instanceof HTMLInputElement)) return;

    if (!input.checked || activeSummaryRead) {
      input.checked = activeSummaryRead;
      return;
    }

    await onMarkReadAndAdvance();
    input.checked = activeSummaryRead;
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
    <label class="read-check">
      <input
        type="checkbox"
        checked={activeSummaryRead || markingRead}
        disabled={markingRead || activeSummaryRead}
        onchange={handleReadCheckboxChange}
        aria-label="Mark summary read and jump to next unread"
      />
      <span>Read</span>
    </label>
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

  .read-check {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    min-width: 112px;
    min-height: 44px;
    padding: var(--space-xs) var(--space-md);
    border-radius: var(--radius-full);
    color: var(--foreground);
    cursor: pointer;
  }
  .read-check:has(input:disabled) {
    cursor: default;
    color: var(--soft-foreground);
  }
  .read-check input {
    width: 18px;
    height: 18px;
    margin: 0;
    accent-color: var(--accent);
    cursor: inherit;
  }
  .read-check span {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  @media (hover: hover) and (pointer: fine) {
    .nav-btn:hover:not(:disabled) {
      color: var(--accent-strong);
    }
    .channel-trigger:hover {
      background: var(--accent-wash);
    }
    .read-check:hover {
      background: var(--accent-wash);
    }
    .read-check:has(input:disabled):hover {
      background: transparent;
    }
  }
</style>
