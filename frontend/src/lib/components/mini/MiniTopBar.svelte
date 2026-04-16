<script lang="ts">
  import FilterIcon from "$lib/components/icons/FilterIcon.svelte";

  interface Props {
    readProgress: number;
    activeIndex: number;
    totalCount: number;
    showCounter: boolean;
    showUnreadOnly: boolean;
    activeFilterCount: number;
    unreadCount: number;
    onToggleFilter: () => void;
  }

  let {
    readProgress,
    activeIndex,
    totalCount,
    showCounter,
    showUnreadOnly,
    activeFilterCount,
    unreadCount,
    onToggleFilter,
  }: Props = $props();
</script>

<header class="mini-bar">
  <div class="bar-left">
    <a class="bar-logo" href="/" data-sveltekit-preload-data="hover">dastill</a>
    <span class="bar-sep"></span>
    <span class="bar-label">mini</span>
  </div>

  <div class="bar-right">
    {#if showCounter}
      <span class="nav-pos"
        >{activeIndex + 1}<span class="nav-pos-sep">/</span>{totalCount}</span
      >
    {/if}

    {#if unreadCount > 0}
      <span class="unread-badge">{unreadCount} unread</span>
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
      {#if activeFilterCount > 0}
        <span class="filter-count" aria-hidden="true">{activeFilterCount}</span>
      {/if}
    </button>
  </div>

  {#if showCounter}
    <div class="progress-track">
      <div
        class="progress-fill"
        style="transform: scaleX({readProgress})"
      ></div>
    </div>
  {/if}
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
    font-variation-settings: "opsz" 72;
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
  .bar-right {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
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
  .unread-badge {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--accent);
    flex-shrink: 0;
  }
  .filter-btn {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-full);
    border: none;
    background: var(--surface);
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
    background: var(--accent-soft);
    color: var(--accent-strong);
  }
  .filter-btn--active:hover {
    background: var(--accent-wash-strong);
  }
  .filter-count {
    position: absolute;
    top: -2px;
    right: -2px;
    display: grid;
    min-width: 16px;
    height: 16px;
    place-items: center;
    border-radius: var(--radius-full);
    background: var(--accent);
    color: white;
    padding: 0 4px;
    font-size: 9px;
    font-weight: 800;
    line-height: 1;
  }
  .progress-track {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: transparent;
  }
  .progress-fill {
    height: 100%;
    background: var(--accent);
    transform-origin: left;
    transition: transform 80ms linear;
    opacity: 0.7;
  }

  @media (min-width: 640px) {
    .mini-bar {
      padding-left: var(--space-lg);
      padding-right: var(--space-lg);
    }
  }
</style>
