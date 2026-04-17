<script lang="ts">
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import type { MiniSummaryItem } from "$lib/transport-types";

  interface Props {
    summaries: MiniSummaryItem[];
    activeVideoId: string | null;
    collapsed?: boolean;
    onSelect: (videoId: string) => void;
  }

  let {
    summaries,
    activeVideoId,
    collapsed = false,
    onSelect,
  }: Props = $props();
  let stripRef = $state<HTMLElement | null>(null);
  let stripHeight = $state(0);

  $effect(() => {
    if (!stripRef) return;
    const el = stripRef;
    const measure = () => {
      stripHeight = el.scrollHeight;
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  });

  $effect(() => {
    if (!activeVideoId || !stripRef) return;
    const el = stripRef.querySelector(`[data-video-id="${activeVideoId}"]`);
    if (!el) return;
    const vertical = getComputedStyle(stripRef).flexDirection === "column";
    el.scrollIntoView({
      behavior: "smooth",
      block: vertical ? "center" : "nearest",
      inline: vertical ? "nearest" : "center",
    });
  });
</script>

{#if summaries.length > 1}
  <div
    class="strip-wrap"
    class:strip-wrap--collapsed={collapsed}
    class:strip-wrap--measured={stripHeight > 0}
    style:--strip-h="{stripHeight}px"
  >
    <nav
      class="strip"
      bind:this={stripRef}
      aria-label="Summary list"
      aria-hidden={collapsed}
    >
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
  </div>
{/if}

<style>
  .strip-wrap {
    flex-shrink: 0;
    overflow: hidden;
  }
  .strip-wrap--measured {
    max-height: var(--strip-h);
    transition:
      max-height 440ms cubic-bezier(0.4, 0, 0.2, 1) 80ms,
      opacity 220ms ease;
    will-change: max-height, opacity;
  }
  .strip-wrap--measured.strip-wrap--collapsed {
    max-height: 0;
    opacity: 0;
    pointer-events: none;
  }
  .strip {
    display: flex;
    gap: var(--space-xs);
    padding: 0 0 var(--space-sm);
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    border-bottom: 1px solid var(--border);
  }
  .strip::-webkit-scrollbar {
    display: none;
  }
  .strip-card {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: 4px var(--space-sm);
    border-radius: 0;
    border: none;
    background: transparent;
    color: var(--soft-foreground);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    text-align: left;
    flex-shrink: 0;
    width: 148px;
    min-height: 40px;
    transition:
      background 160ms ease,
      color 160ms ease;
  }
  .strip-card + .strip-card::before {
    content: "";
    position: absolute;
    left: calc(-1 * var(--space-xs) / 2);
    top: 8px;
    bottom: 8px;
    width: 1px;
    background: var(--border-soft);
    pointer-events: none;
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
    .strip-card {
      width: 196px;
    }
  }

  @media (min-width: 960px) {
    .strip {
      flex-direction: column;
      gap: var(--space-xs);
      width: 300px;
      flex-shrink: 0;
      overflow-x: hidden;
      overflow-y: auto;
      scrollbar-color: var(--border) transparent;
      scrollbar-width: thin;
      padding: 0;
      border-right: 1px solid var(--border);
      border-bottom: none;
    }
    .strip::-webkit-scrollbar {
      display: block;
      width: 8px;
    }
    .strip::-webkit-scrollbar-track {
      background: transparent;
    }
    .strip::-webkit-scrollbar-thumb {
      border-radius: var(--radius-full);
      background: var(--border);
    }
    .strip-wrap,
    .strip-wrap--measured,
    .strip-wrap--measured.strip-wrap--collapsed {
      display: flex;
      flex-direction: column;
      min-height: 0;
      height: 100%;
      overflow: hidden;
      max-height: none;
      opacity: 1;
      pointer-events: auto;
      transition: none;
    }
    .strip {
      flex: 1;
      min-height: 0;
    }
    .strip-card {
      width: 100%;
      min-height: 56px;
      padding: var(--space-sm);
    }
    .strip-card + .strip-card::before {
      left: 10px;
      right: 10px;
      top: calc(-1 * var(--space-xs) / 2);
      bottom: auto;
      width: auto;
      height: 1px;
    }
    .strip-thumb {
      width: 64px;
      height: 36px;
    }
    .strip-title {
      font-size: 13px;
      -webkit-line-clamp: 2;
      line-clamp: 2;
    }
  }
</style>
