<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import type { MiniSummaryItem } from "$lib/transport-types";

  interface Props {
    summaries: MiniSummaryItem[];
    activeVideoId: string | null;
    onSelect: (videoId: string) => void;
  }

  let { summaries, activeVideoId, onSelect }: Props = $props();
  let stripRef = $state<HTMLElement | null>(null);

  $effect(() => {
    if (!activeVideoId || !stripRef) return;
    const el = stripRef.querySelector(`[data-video-id="${activeVideoId}"]`);
    el?.scrollIntoView({
      behavior: "smooth",
      inline: "center",
      block: "nearest",
    });
  });
</script>

{#if summaries.length > 1}
  <nav class="strip" bind:this={stripRef} aria-label="Summary list">
    {#each summaries as summary (summary.video_id)}
      <button
        type="button"
        class="strip-card"
        class:strip-card--active={summary.video_id === activeVideoId}
        class:strip-card--read={summary.read}
        data-video-id={summary.video_id}
        onclick={() => onSelect(summary.video_id)}
      >
        <div class="strip-thumb-wrap">
          {#if summary.thumbnail_url}
            <img
              class="strip-thumb"
              src={summary.thumbnail_url}
              alt=""
              loading="lazy"
            />
          {:else}
            <div class="strip-thumb strip-thumb--empty" aria-hidden="true">
              <span class="strip-thumb-dot"></span>
            </div>
          {/if}
          {#if summary.read}
            <span class="strip-read-mark" aria-label="Read">
              <CheckIcon size={10} strokeWidth={2.8} />
            </span>
          {/if}
        </div>
        <span class="strip-title">{summary.title}</span>
      </button>
    {/each}
  </nav>
{/if}

<style>
  .strip {
    display: flex;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    flex-shrink: 0;
  }
  .strip::-webkit-scrollbar {
    display: none;
  }
  .strip-card {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-sm) var(--space-xs) var(--space-xs);
    border-radius: var(--radius-md);
    border: none;
    background: transparent;
    color: var(--soft-foreground);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    text-align: left;
    flex-shrink: 0;
    width: 148px;
    min-height: 44px;
    transition:
      background 160ms ease,
      color 160ms ease;
  }
  .strip-card:hover {
    background: var(--accent-wash);
    color: var(--foreground);
  }
  .strip-card--active {
    background: var(--accent-wash-strong);
    color: var(--accent-strong);
    font-weight: 700;
  }
  .strip-card--active:hover {
    color: var(--accent-strong);
  }
  .strip-card--read:not(.strip-card--active) {
    opacity: 0.55;
  }
  .strip-thumb-wrap {
    position: relative;
    flex-shrink: 0;
  }
  .strip-thumb {
    width: 48px;
    height: 27px;
    border-radius: var(--radius-sm);
    object-fit: cover;
    display: block;
  }
  .strip-card--read:not(.strip-card--active) .strip-thumb {
    filter: grayscale(0.5);
  }
  .strip-thumb--empty {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--muted);
  }
  .strip-thumb-dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: currentColor;
    opacity: 0.5;
  }
  .strip-read-mark {
    position: absolute;
    bottom: -3px;
    right: -3px;
    display: grid;
    place-items: center;
    width: 14px;
    height: 14px;
    border-radius: var(--radius-full);
    background: var(--background);
    color: var(--accent);
  }
  .strip-title {
    overflow: hidden;
    display: -webkit-box;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-height: 1.35;
  }

  @media (min-width: 640px) {
    .strip {
      padding: var(--space-md) var(--space-lg);
    }
    .strip-card {
      width: 196px;
    }
  }
</style>
