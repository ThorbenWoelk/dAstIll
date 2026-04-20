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
  <path d="m15 4 5 5" />
  <path d="M4 20h4l11.5-11.5a2.121 2.121 0 0 0-3-3L5 16v4Z" />
</svg>
