<script lang="ts">
  let { size = 16, spinning = false }: { size?: number; spinning?: boolean } =
    $props();
</script>

<svg
  class:spinning
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.75"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d="M20 11a8 8 0 1 0-2.34 5.66" />
  <path d="M20 4v7h-7" />
</svg>

<style>
  @media (prefers-reduced-motion: no-preference) {
    .spinning {
      animation: spin 900ms linear infinite;
    }
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
