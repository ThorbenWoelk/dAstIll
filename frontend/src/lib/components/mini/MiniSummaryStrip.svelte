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
    {#each summaries as summary, i}
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
            <div class="strip-thumb strip-thumb--empty">
              <span class="strip-thumb-num">{i + 1}</span>
            </div>
          {/if}
          {#if summary.read}
            <span class="strip-read-mark"><CheckIcon size={10} /></span>
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
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-md);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    border-bottom: 1px solid var(--border-soft);
    flex-shrink: 0;
  }
  .strip::-webkit-scrollbar {
    display: none;
  }
  .strip-card {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs);
    padding-right: var(--space-sm);
    border-radius: var(--radius-md);
    border: none;
    background: var(--surface);
    color: var(--foreground);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    text-align: left;
    flex-shrink: 0;
    width: 140px;
    min-height: 44px;
    transition:
      background 120ms,
      box-shadow 120ms;
  }
  .strip-card:hover {
    background: var(--accent-wash);
  }
  .strip-card--active {
    background: var(--accent-wash-strong);
    font-weight: 700;
  }
  .strip-card--read:not(.strip-card--active) {
    color: var(--soft-foreground);
    opacity: 0.7;
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
  .strip-thumb--empty {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--muted);
    color: var(--soft-foreground);
  }
  .strip-thumb-num {
    font-size: 10px;
    font-weight: 700;
  }
  .strip-read-mark {
    position: absolute;
    bottom: -2px;
    right: -2px;
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-full);
    background: var(--surface);
    color: var(--soft-foreground);
  }
  .strip-title {
    overflow: hidden;
    display: -webkit-box;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-height: 1.3;
  }

  @media (min-width: 640px) {
    .strip {
      padding: var(--space-md) var(--space-lg);
      gap: var(--space-sm);
    }
    .strip-card {
      width: 180px;
    }
  }
</style>
