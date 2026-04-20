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
    if (item.status === "loading") return `${stage} running`;
    if (item.status === "pending") return `${stage} queued`;
    return `${stage} failed`;
  }

  function statusDotClass(status: QueueItem["status"]): string {
    if (status === "failed") return "bg-[var(--danger)]";
    if (status === "loading") return "bg-emerald-500 animate-pulse";
    return "bg-[var(--border)]";
  }

  function statusTextClass(status: QueueItem["status"]): string {
    if (status === "failed") return "text-[var(--danger)]";
    if (status === "loading") return "text-emerald-700";
    return "text-[var(--soft-foreground)]";
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
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path
        d="M7 18a4 4 0 0 1-.88-7.903A5 5 0 1 1 15.9 7.5 4.5 4.5 0 0 1 18 16"
      />
      <path d="M12 13v8" />
      <path d="m8 17 4 4 4-4" />
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
      class="absolute right-0 top-full z-40 mt-2 w-[22rem] origin-top-right overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--surface-strong)] shadow-sm"
      role="dialog"
      aria-label="Operations queue"
    >
      <div
        class="flex items-center justify-between border-b border-[var(--border)] bg-[var(--muted)] px-4 py-3"
      >
        <div class="flex items-center gap-2">
          <h3
            class="text-xs font-semibold uppercase tracking-wider text-[var(--soft-foreground)]"
          >
            Operations Queue
          </h3>
          {#if counts.active > 0}
            <span
              class="flex items-center gap-1.5 text-xs font-medium text-[var(--soft-foreground)]"
            >
              <span
                class="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-500"
              ></span>
              Processing {counts.active}
            </span>
          {/if}
        </div>
        <button
          type="button"
          class="inline-flex h-6 w-6 items-center justify-center rounded-md text-[var(--soft-foreground)] transition-colors hover:bg-[var(--surface-strong)] hover:text-[var(--foreground)] disabled:opacity-50"
          onclick={() => void loadQueue()}
          aria-label="Refresh queue"
          disabled={loading}
        >
          <RefreshIcon size={14} />
        </button>
      </div>

      <div class="max-h-[24rem] overflow-y-auto">
        {#if loading && items.length === 0}
          <p class="px-4 py-6 text-sm text-[var(--soft-foreground)]">
            Loading…
          </p>
        {:else if error}
          <p class="px-4 py-6 text-sm text-[var(--danger)]">{error}</p>
        {:else if items.length === 0}
          <p class="px-4 py-6 text-sm text-[var(--soft-foreground)]">
            Nothing in the queue.
          </p>
        {:else}
          <div class="divide-y divide-[var(--border)]">
            {#each items as item (item.video.id + item.stage)}
              <div
                class="grid grid-cols-12 items-center gap-3 px-4 py-3 {item.status ===
                'failed'
                  ? 'bg-[var(--danger)]/5'
                  : ''}"
              >
                <div class="col-span-6 flex items-center gap-2.5 min-w-0">
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="shrink-0 text-[var(--soft-foreground)]"
                    aria-hidden="true"
                  >
                    <rect x="2" y="6" width="14" height="12" rx="2" />
                    <path d="m22 8-6 4 6 4z" />
                  </svg>
                  <span
                    class="truncate text-sm font-medium text-[var(--foreground)]"
                    title={item.video.title}
                  >
                    {item.video.title}
                  </span>
                </div>
                <div class="col-span-4 flex items-center gap-2 min-w-0">
                  <span
                    class="h-1.5 w-1.5 shrink-0 rounded-full {statusDotClass(
                      item.status,
                    )}"
                    aria-hidden="true"
                  ></span>
                  <span
                    class="truncate text-xs font-medium {statusTextClass(
                      item.status,
                    )}"
                  >
                    {statusLabel(item)}
                  </span>
                </div>
                <div class="col-span-2 text-right">
                  {#if item.status === "failed"}
                    <button
                      type="button"
                      class="rounded bg-[var(--muted)] px-2 py-1 text-xs font-medium text-[var(--foreground)] transition-colors hover:bg-[var(--border)] disabled:opacity-50"
                      disabled={retryingVideoIds.has(item.video.id)}
                      onclick={() => void retryItem(item)}
                    >
                      {retryingVideoIds.has(item.video.id) ? "…" : "Retry"}
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
