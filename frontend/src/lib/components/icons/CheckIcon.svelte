<script lang="ts">
  let {
    size = 14,
    strokeWidth = 2.4,
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
  <path d="m5 12 4.5 4.5L19 7"></path>
</svg>
