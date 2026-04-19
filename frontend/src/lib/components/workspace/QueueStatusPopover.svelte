<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { ensureTranscript, listChannels, listVideos } from "$lib/api";
  import type { Channel, Video } from "$lib/types";
  import RefreshIcon from "$lib/components/icons/RefreshIcon.svelte";

  type QueueItem = {
    video: Video;
    channel: Channel | null;
    stage: "transcript" | "summary";
    status: "loading" | "pending" | "failed";
  };

  let open = $state(false);
  let loading = $state(false);
  let items = $state<QueueItem[]>([]);
  let error = $state<string | null>(null);
  const retryingVideoIds = new SvelteSet<string>();
  let container = $state<HTMLDivElement | null>(null);

  const counts = $derived.by(() => {
    let active = 0;
    let failed = 0;
    for (const item of items) {
      if (item.status === "failed") failed += 1;
      else active += 1;
    }
    return { active, failed, total: items.length };
  });

  function extractQueueItems(
    videos: Video[],
    channel: Channel | null,
  ): QueueItem[] {
    const results: QueueItem[] = [];
    for (const video of videos) {
      const t = video.transcript_status;
      if (t === "loading" || t === "pending" || t === "failed") {
        results.push({ video, channel, stage: "transcript", status: t });
        continue;
      }
      const s = video.summary_status;
      if (s === "loading" || s === "pending" || s === "failed") {
        results.push({ video, channel, stage: "summary", status: s });
      }
    }
    return results;
  }

  async function loadQueue() {
    loading = true;
    error = null;
    try {
      const channels = await listChannels();
      const perChannel = await Promise.all(
        channels.map(async (channel) => {
          try {
            const page = await listVideos(
              channel.id,
              25,
              0,
              "all",
              undefined,
              true,
              true,
            );
            return extractQueueItems(page.videos, channel);
          } catch {
            return [] as QueueItem[];
          }
        }),
      );
      const merged = perChannel.flat();
      merged.sort((a, b) => {
        const order = { failed: 0, loading: 1, pending: 2 } as const;
        if (order[a.status] !== order[b.status]) {
          return order[a.status] - order[b.status];
        }
        return b.video.published_at.localeCompare(a.video.published_at);
      });
      items = merged;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load queue";
      items = [];
    } finally {
      loading = false;
    }
  }

  async function toggleOpen() {
    const next = !open;
    open = next;
    if (next) await loadQueue();
  }

  async function retryItem(item: QueueItem) {
    const id = item.video.id;
    if (retryingVideoIds.has(id)) return;
    retryingVideoIds.add(id);
    try {
      await ensureTranscript(id);
    } catch {
      // ignore; reload will show latest status
    } finally {
      retryingVideoIds.delete(id);
      await loadQueue();
    }
  }

  function handleDocumentClick(event: MouseEvent) {
    if (!open || !container) return;
    const target = event.target as Node | null;
    if (target && !container.contains(target)) {
      open = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (open && event.key === "Escape") {
      open = false;
    }
  }

  onMount(() => {
    document.addEventListener("click", handleDocumentClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("click", handleDocumentClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  });

  function statusLabel(item: QueueItem): string {
    const stage = item.stage === "transcript" ? "Transcript" : "Summary";
    if (item.status === "loading") return `${stage} · running`;
    if (item.status === "pending") return `${stage} · queued`;
    return `${stage} · failed`;
  }
</script>

<div bind:this={container} class="relative">
  <button
    type="button"
    class="flex h-8 items-center gap-1.5 rounded-md border border-[var(--border-soft)] bg-[var(--surface)] px-2.5 text-[12px] font-medium text-[var(--soft-foreground)] transition-colors duration-150 hover:bg-[var(--surface-strong)] hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40"
    aria-expanded={open}
    aria-haspopup="true"
    onclick={() => void toggleOpen()}
    title="Queue status"
  >
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.75"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="10" />
      <polyline points="12 6 12 12 16 14" />
    </svg>
    <span>Queue</span>
    {#if counts.total > 0}
      <span
        class={`ml-0.5 inline-flex h-4 min-w-4 items-center justify-center rounded px-1 text-[10px] font-semibold ${counts.failed > 0 ? "bg-[var(--danger)]/15 text-[var(--danger)]" : "bg-[var(--accent-soft)] text-[var(--accent-strong)]"}`}
      >
        {counts.total}
      </span>
    {/if}
  </button>

  {#if open}
    <div
      class="absolute right-0 top-full z-40 mt-2 w-80 origin-top-right rounded-md border border-[var(--border-soft)] bg-[var(--background)] shadow-lg"
      role="dialog"
      aria-label="Queue status"
    >
      <div
        class="flex items-center justify-between border-b border-[var(--border-soft)] px-3 py-2"
      >
        <p class="text-[12px] font-semibold text-[var(--foreground)]">Queue</p>
        <button
          type="button"
          class="inline-flex h-6 w-6 items-center justify-center rounded-md text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface)] hover:text-[var(--foreground)]"
          onclick={() => void loadQueue()}
          aria-label="Refresh queue"
          disabled={loading}
        >
          <RefreshIcon size={14} />
        </button>
      </div>

      <div class="max-h-[22rem] overflow-y-auto py-1">
        {#if loading && items.length === 0}
          <p class="px-3 py-4 text-[12px] text-[var(--soft-foreground)]">
            Loading…
          </p>
        {:else if error}
          <p class="px-3 py-4 text-[12px] text-[var(--danger)]">{error}</p>
        {:else if items.length === 0}
          <p class="px-3 py-4 text-[12px] text-[var(--soft-foreground)]">
            Nothing in the queue.
          </p>
        {:else}
          {#each items as item (item.video.id + item.stage)}
            <div class="flex items-start gap-2 px-3 py-2">
              <div class="min-w-0 flex-1">
                <p
                  class="truncate text-[12px] font-medium text-[var(--foreground)]"
                  title={item.video.title}
                >
                  {item.video.title}
                </p>
                <p
                  class="mt-0.5 flex items-center gap-1.5 text-[11px] text-[var(--soft-foreground)]"
                >
                  {#if item.channel}
                    <span class="truncate">{item.channel.name}</span>
                    <span
                      class="h-1 w-1 shrink-0 rounded-full bg-[var(--border)]"
                      aria-hidden="true"
                    ></span>
                  {/if}
                  <span
                    class={item.status === "failed"
                      ? "text-[var(--danger)]"
                      : ""}
                  >
                    {statusLabel(item)}
                  </span>
                </p>
              </div>
              {#if item.status === "failed"}
                <button
                  type="button"
                  class="shrink-0 rounded-md border border-[var(--border-soft)] px-2 py-0.5 text-[11px] font-medium text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface)] hover:text-[var(--foreground)] disabled:opacity-50"
                  disabled={retryingVideoIds.has(item.video.id)}
                  onclick={() => void retryItem(item)}
                >
                  {retryingVideoIds.has(item.video.id) ? "Retrying…" : "Retry"}
                </button>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>
