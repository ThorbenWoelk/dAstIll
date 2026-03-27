<script lang="ts">
  let {
    label,
    compact = false,
  }: {
    label: string;
    compact?: boolean;
  } = $props();
</script>

<span
  class={`inline-flex max-w-full items-center rounded-full bg-[var(--accent-wash)] text-[var(--foreground)] ${
    compact
      ? "h-5 shrink-0 px-1.5 py-0 text-[14px] leading-5"
      : "px-2.5 py-1 text-[11px] leading-none"
  }`}
>
  <span class="truncate">{label}</span>
</span>
