<script lang="ts">
  let {
    size = 16,
    strokeWidth = 2.2,
    className = "",
  }: { size?: number; strokeWidth?: number; className?: string } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  aria-hidden="true"
>
  <line x1="3" y1="6" x2="21" y2="6" />
  <line x1="7" y1="12" x2="17" y2="12" />
  <line x1="10" y1="18" x2="14" y2="18" />
</svg>
