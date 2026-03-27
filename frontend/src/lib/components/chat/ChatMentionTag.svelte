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
      ? "px-1.5 py-0.5 text-[12px] leading-[1.2]"
      : "px-2.5 py-1 text-[11px] leading-none"
  }`}
>
  <span class="truncate">{label}</span>
</span>
