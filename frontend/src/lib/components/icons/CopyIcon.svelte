<script lang="ts">
  type Props = {
    size?: number | string;
    strokeWidth?: number;
    class?: string;
  };

  let { size = 14, strokeWidth = 2, class: className = "" }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
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
  <rect x="8" y="8" width="12" height="12" rx="2" />
  <path d="M4 16V6a2 2 0 0 1 2-2h10" />
</svg>
