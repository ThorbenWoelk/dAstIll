<script lang="ts">
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import FilterIcon from "$lib/components/icons/FilterIcon.svelte";

  interface Props {
    activeIndex: number;
    totalCount: number;
    showCounter: boolean;
    showUnreadOnly: boolean;
    activeFilterCount: number;
    unreadCount: number;
    onToggleFilter: () => void;
    channelName?: string | null;
    canPickChannel?: boolean;
    onOpenChannelPicker?: () => void;
  }

  let {
    activeIndex,
    totalCount,
    showCounter,
    showUnreadOnly,
    activeFilterCount,
    unreadCount,
    onToggleFilter,
    channelName = null,
    canPickChannel = false,
    onOpenChannelPicker,
  }: Props = $props();

  const filterBadgeCount = $derived(
    showUnreadOnly && unreadCount > 0
      ? unreadCount
      : activeFilterCount > 0
        ? activeFilterCount
        : 0,
  );
</script>

<header class="mini-bar">
  <a class="bar-logo" href="/" data-sveltekit-preload-data="hover">
    <span class="bar-logo-word">dastill</span>
    <span class="bar-logo-mini">mini</span>
  </a>

  {#if canPickChannel && onOpenChannelPicker}
    <button
      type="button"
      class="bar-channel"
      onclick={onOpenChannelPicker}
      aria-label="Change channel"
    >
      <span class="bar-channel-name">{channelName ?? "All channels"}</span>
      <ChevronIcon direction="down" size={10} strokeWidth={2.4} />
    </button>
  {/if}

  <div class="bar-right">
    {#if showCounter}
      <span class="nav-pos" aria-label="Position">
        <span class="nav-pos-num">{activeIndex + 1}</span>
        <span class="nav-pos-total">of {totalCount}</span>
      </span>
    {/if}

    <button
      type="button"
      class="filter-btn"
      class:filter-btn--active={showUnreadOnly}
      onclick={onToggleFilter}
      aria-label={showUnreadOnly ? "Show all summaries" : "Show unread only"}
      data-tooltip={showUnreadOnly ? "Show all" : "Unread only"}
      data-tooltip-placement="bottom"
    >
      <FilterIcon size={15} strokeWidth={2.3} />
      {#if filterBadgeCount > 0}
        <span class="filter-count" aria-hidden="true">{filterBadgeCount}</span>
      {/if}
    </button>
  </div>
</header>

<style>
  .mini-bar {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-md);
    padding: max(var(--space-sm), env(safe-area-inset-top)) var(--space-md)
      var(--space-sm);
    flex-shrink: 0;
    min-height: 48px;
    border-bottom: 3px double var(--foreground);
  }
  .bar-logo {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    color: var(--foreground);
    text-decoration: none;
    flex-shrink: 0;
  }
  .bar-logo-word {
    font-family: "Fraunces", serif;
    font-size: 18px;
    font-weight: 600;
    letter-spacing: -0.02em;
    font-variation-settings: "opsz" 72;
  }
  .bar-logo-mini {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--soft-foreground);
    transition: color 120ms;
  }
  .bar-logo {
    transition: opacity 120ms;
  }
  .bar-logo:hover {
    opacity: 0.7;
  }
  .bar-right {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    flex-shrink: 0;
    margin-left: auto;
  }
  .bar-channel {
    display: none;
    align-items: center;
    gap: var(--space-xs);
    height: 32px;
    padding: 0 var(--space-sm) 0 var(--space-md);
    border-radius: var(--radius-full);
    border: none;
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
    min-width: 0;
    max-width: 360px;
    transition: background 120ms;
  }
  .bar-channel:hover {
    background: var(--accent-wash);
  }
  .bar-channel-name {
    font-size: 13px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .nav-pos {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    font-variant-numeric: tabular-nums;
  }
  .nav-pos-num {
    font-family: "Fraunces", serif;
    font-size: 15px;
    font-weight: 600;
    color: var(--foreground);
    letter-spacing: -0.01em;
    font-variation-settings: "opsz" 72;
  }
  .nav-pos-total {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--soft-foreground);
  }
  .filter-btn {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-full);
    border: none;
    background: transparent;
    color: var(--foreground);
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
    flex-shrink: 0;
  }
  .filter-btn:hover {
    background: var(--accent-wash);
  }
  .filter-btn--active {
    color: var(--accent-strong);
    background: var(--accent-soft);
  }
  .filter-btn--active:hover {
    background: var(--accent-wash-strong);
  }
  .filter-count {
    position: absolute;
    top: -1px;
    right: -1px;
    display: grid;
    min-width: 16px;
    height: 16px;
    place-items: center;
    border-radius: var(--radius-full);
    background: var(--accent);
    color: var(--background);
    padding: 0 4px;
    font-size: 9px;
    font-weight: 800;
    line-height: 1;
    box-shadow: 0 0 0 2px var(--background);
  }
  @media (min-width: 640px) {
    .mini-bar {
      padding-left: var(--space-lg);
      padding-right: var(--space-lg);
    }
  }
  @media (min-width: 960px) {
    .bar-channel {
      display: inline-flex;
    }
  }
</style>
