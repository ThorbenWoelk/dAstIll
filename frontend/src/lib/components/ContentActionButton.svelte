<script lang="ts">
  type ContentActionIcon =
    | "edit"
    | "format"
    | "regenerate"
    | "revert"
    | "youtube";

  let {
    icon,
    compact = false,
    disabled = false,
    loading = false,
    label = "",
    tooltip = "",
    tooltipAnchor = undefined,
    tooltipPlacement = "bottom",
    href = null,
    onClick = () => {},
  }: {
    icon: ContentActionIcon;
    compact?: boolean;
    disabled?: boolean;
    loading?: boolean;
    label?: string;
    tooltip?: string;
    tooltipAnchor?: string | undefined;
    tooltipPlacement?: string | undefined;
    href?: string | null;
    onClick?: () => void;
  } = $props();

  const outlinedButtonClass =
    "inline-flex h-10 w-10 items-center justify-center rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--background)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--accent)] hover:shadow-sm disabled:cursor-not-allowed disabled:opacity-30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40";
  const ghostButtonClass =
    "inline-flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] border border-transparent text-[var(--soft-foreground)] transition-all hover:border-[var(--border-soft)] hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] disabled:cursor-not-allowed disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40";

  let buttonClass = $derived(compact ? ghostButtonClass : outlinedButtonClass);
  let buttonStateClass = $derived(loading && !disabled ? "cursor-wait" : "");
  let strokeWidth = $derived(compact ? "2.2" : "2.5");
</script>

{#if href}
  <a
    {href}
    target="_blank"
    rel="noopener noreferrer"
    class={buttonClass}
    aria-label={label}
    data-tooltip={tooltip}
    data-tooltip-anchor={tooltipAnchor}
    data-tooltip-placement={tooltipPlacement}
  >
    <svg viewBox="0 0 24 24" class="h-4 w-4" aria-hidden="true">
      <rect
        x="3"
        y="6"
        width="18"
        height="12"
        rx="2"
        ry="2"
        fill="none"
        stroke="currentColor"
        stroke-width={strokeWidth}
      />
      <path d="M10 9l5 3-5 3V9z" fill="currentColor" />
    </svg>
  </a>
{:else}
  <button
    type="button"
    class={`${buttonClass} ${buttonStateClass}`}
    onclick={onClick}
    {disabled}
    aria-label={label}
    data-tooltip={tooltip}
    data-tooltip-anchor={tooltipAnchor}
    data-tooltip-placement={tooltipPlacement}
  >
    {#if loading}
      <svg viewBox="0 0 24 24" class="h-4 w-4 animate-spin" aria-hidden="true">
        <circle
          cx="12"
          cy="12"
          r="9"
          fill="none"
          stroke="currentColor"
          stroke-opacity="0.25"
          stroke-width="2"
        />
        <path
          d="M12 3a9 9 0 0 1 9 9"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        />
      </svg>
    {:else if icon === "format"}
      <svg viewBox="0 0 16 16" class="h-4 w-4" aria-hidden="true">
        <path
          d="M3 4h10M3 8h6M3 12h8"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-width={strokeWidth}
        />
      </svg>
    {:else if icon === "revert"}
      <svg viewBox="0 0 24 24" class="h-4 w-4" aria-hidden="true">
        <path
          d="M9 15 3 9m0 0 6-6M3 9h10.5a5.5 5.5 0 0 1 0 11H9"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width={strokeWidth}
        />
      </svg>
    {:else if icon === "regenerate"}
      <svg
        viewBox="0 0 24 24"
        class="h-4 w-4"
        fill="none"
        stroke="currentColor"
        stroke-width={strokeWidth}
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" />
        <path d="M21 3v5h-5" />
      </svg>
    {:else}
      <svg
        width="15"
        height="15"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width={strokeWidth}
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M12 20h9" />
        <path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" />
      </svg>
    {/if}
  </button>
{/if}
