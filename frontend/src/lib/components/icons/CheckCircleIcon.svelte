<script lang="ts">
  let {
    size = 20,
    strokeWidth = 1.8,
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
  <circle cx="12" cy="12" r="9" />
  <path d="m8.5 12.2 2.4 2.4 4.9-5.2" />
</svg>
