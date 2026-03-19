<script lang="ts">
  let {
    size = 14,
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
  <path d="M14 5h5v5"></path>
  <path d="M10 14 19 5"></path>
  <path d="M19 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h4"></path>
</svg>
