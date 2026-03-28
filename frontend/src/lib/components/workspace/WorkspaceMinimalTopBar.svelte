<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    trailing,
  }: {
    title?: string;
    trailing?: Snippet;
  } = $props();
</script>

<div class="flex min-w-0 flex-1 items-center gap-3">
  {#if title}
    <h2
      class="min-w-0 truncate font-serif text-[18px] font-bold tracking-tight text-[var(--foreground)]"
    >
      {title}
    </h2>
  {/if}
</div>
{#if trailing}
  {@render trailing()}
{/if}
