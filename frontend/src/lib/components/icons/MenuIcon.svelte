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
  <path d="M4 6h16" />
  <path d="M4 12h16" />
  <path d="M4 18h16" />
</svg>
