<script lang="ts">
  let { message = "" }: { message?: string } = $props();
</script>

<div class="mt-4 space-y-8 animate-pulse" role="status" aria-live="polite">
  <div class="h-10 w-3/5 rounded-[var(--radius-sm)] bg-[var(--muted)]/60"></div>
  <div class="space-y-4 pt-4">
    <div class="h-4 w-full rounded-full bg-[var(--muted)]/50"></div>
    <div class="h-4 w-11/12 rounded-full bg-[var(--muted)]/50"></div>
    <div class="h-4 w-10/12 rounded-full bg-[var(--muted)]/50"></div>
    <div class="h-4 w-full rounded-full bg-[var(--muted)]/50"></div>
    <div class="h-4 w-3/4 rounded-full bg-[var(--muted)]/50"></div>
  </div>
  {#if message}
    <p
      class="pt-10 text-center text-[10px] font-bold uppercase tracking-[0.4em] text-[var(--accent)]"
    >
      {message}
    </p>
  {/if}
</div>
