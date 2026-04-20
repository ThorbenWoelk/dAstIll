<script lang="ts">
  type Props = {
    size?: number | string;
    strokeWidth?: number;
    class?: string;
  };

  let { size = 14, strokeWidth = 1.5, class: className = "" }: Props = $props();
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
  <rect x="3" y="3" width="18" height="18" rx="3" />
  <path d="M7 9h10" />
  <path d="M7 13h10" />
  <path d="M7 17h6" />
</svg>
