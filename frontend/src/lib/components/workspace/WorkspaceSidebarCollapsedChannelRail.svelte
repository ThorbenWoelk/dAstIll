<script lang="ts">
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import type { Channel } from "$lib/types";

  let {
    loadingChannels,
    filteredChannels,
    selectedChannelId,
    onToggleCollapse,
    onSelectChannel,
  }: {
    loadingChannels: boolean;
    filteredChannels: Channel[];
    selectedChannelId: string | null;
    onToggleCollapse: () => void;
    onSelectChannel: (channelId: string) => void;
  } = $props();
</script>

<div class="flex items-center justify-center px-2 pb-1 pt-3">
  <button
    type="button"
    class="inline-flex h-7 w-7 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100"
    onclick={onToggleCollapse}
    aria-label="Expand channel sidebar"
  >
    <ChevronIcon direction="right" />
  </button>
</div>

<div
  class="custom-scrollbar mt-1 flex min-h-0 flex-1 flex-col items-center gap-2 overflow-y-auto px-2 pb-4"
>
  {#if loadingChannels}
    {#each Array.from({ length: 5 }) as _, i (i)}
      <div
        class="h-8 w-8 animate-pulse rounded-full bg-[var(--border)] opacity-60"
      ></div>
    {/each}
  {:else}
    {#each filteredChannels as channel (channel.id)}
      <button
        type="button"
        class={`relative flex h-10 w-10 shrink-0 items-center justify-center rounded-full p-0.5 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${selectedChannelId === channel.id ? "bg-[var(--accent-soft)]/60" : "hover:bg-[var(--accent-wash)]"}`}
        onclick={() => onSelectChannel(channel.id)}
        data-tooltip={channel.name}
        data-tooltip-placement="right"
        aria-label={channel.name}
      >
        <span
          class={`flex h-full w-full items-center justify-center overflow-hidden rounded-full bg-[var(--muted)] ${selectedChannelId === channel.id ? "ring-1 ring-[var(--accent)]/20" : ""}`}
        >
          {#if channel.thumbnail_url}
            <img
              src={channel.thumbnail_url}
              alt={channel.name}
              class="h-full w-full object-cover"
              referrerpolicy="no-referrer"
            />
          {:else}
            <span
              class="flex h-full w-full items-center justify-center text-[10px] font-bold text-[var(--soft-foreground)]"
              >{channel.name.charAt(0)}</span
            >
          {/if}
        </span>
      </button>
    {/each}
  {/if}
</div>
