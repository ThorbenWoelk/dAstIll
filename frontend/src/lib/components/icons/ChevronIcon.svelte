<script lang="ts">
  let {
    direction = "left",
    size = 10,
    strokeWidth = 2,
    className = "",
  }: {
    direction?: "left" | "right" | "down";
    size?: number;
    strokeWidth?: number;
    className?: string;
  } = $props();

  let path = $derived(
    direction === "right"
      ? "M9 6l6 6-6 6"
      : direction === "down"
        ? "M6 9l6 6 6-6"
        : "M15 6l-6 6 6 6",
  );
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
  <path d={path}></path>
</svg>
