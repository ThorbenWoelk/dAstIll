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
  <circle cx="11" cy="11" r="7"></circle>
  <path d="m20 20-3.5-3.5"></path>
</svg>
