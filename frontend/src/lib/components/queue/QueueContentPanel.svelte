<script lang="ts">
  import Toggle from "$lib/components/Toggle.svelte";
  import { formatShortDate } from "$lib/utils/date";
  import { formatSyncDate } from "$lib/workspace/content";
  import type { QueueTab } from "$lib/types";
  import type {
    QueueContentPanelActions,
    QueueContentPanelState,
  } from "$lib/workspace/component-props";
  import { swipeBack } from "$lib/mobile-shell/swipe";

  let {
    readOnly = false,
    state: panelState = {
      mobileVisible: false,
      selectedChannel: null,
      selectedChannelId: null,
      queueTab: "transcripts",
      queueStats: { total: 0, loading: 0, pending: 0, failed: 0 },
      failedTranscriptVideos: [],
      retryingTranscriptVideoId: null,
      effectiveEarliestSyncDate: null,
      earliestSyncDateInput: "",
      savingSyncDate: false,
      refreshingChannel: false,
    },
    actions = {
      onBack: () => {},
      onQueueTabChange: () => {},
      onSaveSyncDate: async () => {},
      onRetryTranscript: async (_videoId: string) => {},
    },
  }: {
    readOnly?: boolean;
    state?: QueueContentPanelState;
    actions?: QueueContentPanelActions;
  } = $props();

  let localSyncDateInput = $state("");

  const queueTabCopy = {
    transcripts: {
      eyebrow: "Transcript queue",
      title: "Monitor extraction progress",
      detail:
        "Track videos waiting for transcript extraction, active jobs, and failures for this channel.",
    },
    summaries: {
      eyebrow: "Summary queue",
      title: "Monitor summary generation",
      detail:
        "Track which transcript-ready videos are moving through summary generation and which ones need attention.",
    },
    evaluations: {
      eyebrow: "Evaluation queue",
      title: "Monitor quality scoring",
      detail:
        "Track summaries waiting for quality evaluation so you can see what still needs scoring.",
    },
  } satisfies Record<
    QueueTab,
    { eyebrow: string; title: string; detail: string }
  >;

  $effect(() => {
    localSyncDateInput = panelState.earliestSyncDateInput;
  });

  async function saveSyncDate() {
    await actions.onSaveSyncDate(localSyncDateInput);
  }

  function retryTranscript(videoId: string) {
    return actions.onRetryTranscript?.(videoId);
  }

  function handleQueueTabChange(value: string) {
    return actions.onQueueTabChange(value as QueueTab);
  }
</script>

<section
  class={`fade-in stagger-3 relative z-10 flex min-h-0 min-w-0 flex-col overflow-visible border-0 lg:sticky lg:top-4 lg:h-[calc(100vh-4rem)] lg:gap-4 lg:pb-6 lg:pl-5 ${panelState.mobileVisible ? "h-full" : "hidden lg:flex"}`}
  id="content-view"
>
  <div class="flex flex-col gap-3 px-4 max-lg:pb-1 max-lg:pt-3 sm:px-6 lg:px-0">
    <div class="flex items-center justify-between gap-3">
      <h2 class="text-base font-bold tracking-tight text-[var(--foreground)]">
        Status
      </h2>
      {#if panelState.refreshingChannel}
        <span
          class="h-3 w-3 animate-spin rounded-full border-2 border-[var(--border)] border-t-[var(--accent)]"
          role="status"
          aria-label="Refreshing queue status"
        ></span>
      {/if}
    </div>
    <div
      class="flex flex-col gap-2 border-b border-[var(--border-soft)] pb-3 lg:flex-row lg:items-end lg:justify-between"
    >
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2 lg:hidden">
          <button
            type="button"
            class="inline-flex items-center gap-2 text-[12px] font-semibold text-[var(--foreground)] opacity-80"
            onclick={actions.onBack}
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.6"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="15 18 9 12 15 6" />
            </svg>
            Back to queue
          </button>
        </div>
        <p
          class="text-[10px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-55"
        >
          {queueTabCopy[panelState.queueTab].eyebrow}
        </p>
        <p
          class="mt-1 text-[20px] font-semibold tracking-tight text-[var(--foreground)]"
        >
          {panelState.selectedChannel
            ? panelState.selectedChannel.name
            : "Queue overview"}
        </p>
        <p
          class="mt-2 max-w-[34rem] text-[14px] leading-6 text-[var(--soft-foreground)]"
        >
          {queueTabCopy[panelState.queueTab].detail}
        </p>
      </div>

      <div id="queue-stage-tabs" class="min-w-0 lg:w-auto lg:min-w-[18rem]">
        <Toggle
          ariaLabel="Queue tabs"
          options={["transcripts", "summaries", "evaluations"]}
          value={panelState.queueTab}
          showIcons={false}
          labels={{
            transcripts: "Transcripts",
            summaries: "Summaries",
            evaluations: "Evals",
          }}
          onChange={handleQueueTabChange}
        />
      </div>
    </div>
  </div>

  <div
    class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-4 max-lg:pt-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0"
    role="region"
    aria-label="Queue content panel"
    use:swipeBack={{
      enabled: panelState.mobileVisible,
      onBack: actions.onBack,
    }}
  >
    {#if !panelState.selectedChannelId}
      <div
        class="flex h-full flex-col items-center justify-center py-20 text-center"
      >
        <div
          class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-[var(--accent-soft)]/60 text-[var(--accent-strong)]"
        >
          <svg
            width="22"
            height="22"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <rect x="3" y="4" width="6" height="16" rx="1.5" />
            <rect x="10" y="4" width="5" height="16" rx="1.5" />
            <rect x="16" y="4" width="5" height="16" rx="1.5" />
          </svg>
        </div>
        <p class="mt-4 text-[17px] font-semibold text-[var(--foreground)]">
          Select a channel
        </p>
        <p
          class="mt-2 max-w-[22rem] text-[14px] leading-6 text-[var(--soft-foreground)]"
        >
          Choose a channel to inspect queue health, sync depth, and current
          processing activity.
        </p>
      </div>
    {:else}
      <div class="flex flex-col gap-8 pb-24">
        <!-- Stats row -->
        <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5">
          <span class="text-[14px] font-semibold text-[var(--foreground)]">
            {panelState.queueStats.total} items
          </span>
          <span class="text-[14px] text-[var(--soft-foreground)]">
            {panelState.queueStats.pending} waiting
          </span>
          {#if panelState.queueStats.loading > 0}
            <span
              class="rounded-full bg-amber-500/10 px-2 py-1 text-[11px] font-semibold text-amber-700 dark:text-amber-400"
            >
              {panelState.queueStats.loading} active
            </span>
          {/if}
          {#if panelState.queueStats.failed > 0}
            <span
              class="rounded-full bg-[var(--danger-soft)] px-2 py-1 text-[11px] font-semibold text-[var(--danger-foreground)]"
            >
              {panelState.queueStats.failed} failed
            </span>
          {/if}
        </div>

        <!-- Stage info -->
        <div>
          <p
            class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
          >
            Current stage
          </p>
          <p
            class="mt-2 text-[17px] font-semibold tracking-tight text-[var(--foreground)]"
          >
            {queueTabCopy[panelState.queueTab].title}
          </p>

          <div class="mt-4 grid grid-cols-2 gap-x-6 gap-y-4">
            <div>
              <p
                class="mb-1 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
              >
                Sync from
              </p>
              <p class="text-[14px] font-semibold text-[var(--foreground)]">
                {formatSyncDate(panelState.effectiveEarliestSyncDate)}
              </p>
            </div>
          </div>

          {#if panelState.refreshingChannel}
            <p
              class="mt-4 flex items-center gap-2 text-[12px] text-[var(--soft-foreground)]"
            >
              <span
                class="h-3 w-3 shrink-0 animate-spin rounded-full border-2 border-[var(--border)] border-t-[var(--accent)]"
              ></span>
              Refreshing channel state in the background.
            </p>
          {/if}

          {#if panelState.queueStats.total === 0}
            <p
              class="mt-4 text-[12px] text-[var(--soft-foreground)] opacity-70"
            >
              Everything for this stage is currently clear.
            </p>
          {/if}
        </div>

        <!-- Sync depth -->
        <div class="border-t border-[var(--border-soft)] pt-6">
          <p
            class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
          >
            Sync depth
          </p>
          <p class="mt-2 text-[14px] font-semibold text-[var(--foreground)]">
            Control how far back this queue should sync.
          </p>
          {#if !readOnly}
            <div class="mt-3 flex items-center gap-2">
              <input
                type="date"
                class="min-w-0 flex-1 rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-transparent px-3 py-2 text-[12px] font-medium transition-colors focus:border-[var(--accent)]/40 focus:outline-none"
                bind:value={localSyncDateInput}
                disabled={panelState.savingSyncDate}
              />
              <button
                type="button"
                class="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
                onclick={() => void saveSyncDate()}
                disabled={!localSyncDateInput || panelState.savingSyncDate}
              >
                {panelState.savingSyncDate ? "..." : "Set"}
              </button>
            </div>
          {/if}
          <p class="mt-2 text-[12px] text-[var(--soft-foreground)]">
            Current boundary: {formatSyncDate(
              panelState.effectiveEarliestSyncDate,
            )}.
          </p>
        </div>

        <!-- Failed downloads -->
        {#if panelState.queueTab === "transcripts" && panelState.failedTranscriptVideos && panelState.failedTranscriptVideos.length > 0}
          <div class="border-t border-[var(--border-soft)] pt-6">
            <div class="flex items-center justify-between gap-3">
              <p
                class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
              >
                Failed downloads
              </p>
              <span
                class="rounded-full bg-[var(--danger-soft)] px-2 py-1 text-[11px] font-semibold text-[var(--danger-foreground)]"
              >
                {panelState.failedTranscriptVideos.length} failed
              </span>
            </div>
            <p class="mt-2 text-[14px] font-semibold text-[var(--foreground)]">
              Retry transcript extraction
            </p>

            <div class="mt-4 space-y-0">
              {#each panelState.failedTranscriptVideos as video (video.id)}
                <div
                  class="flex flex-col gap-3 border-t border-[var(--border-soft)] py-4 first:border-t-0 sm:flex-row sm:items-center sm:justify-between"
                >
                  <div class="min-w-0">
                    <p
                      class="line-clamp-2 text-[14px] font-semibold text-[var(--foreground)]"
                    >
                      {video.title}
                    </p>
                    <p class="mt-1 text-[12px] text-[var(--soft-foreground)]">
                      Published {formatShortDate(video.published_at)}
                    </p>
                  </div>

                  {#if !readOnly}
                    <button
                      type="button"
                      class="inline-flex shrink-0 items-center justify-center rounded-full bg-[var(--foreground)] px-4 py-2 text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--background)] transition-all hover:bg-[var(--accent-strong)] disabled:opacity-40"
                      onclick={() => void retryTranscript(video.id)}
                      disabled={panelState.retryingTranscriptVideoId ===
                        video.id}
                    >
                      {panelState.retryingTranscriptVideoId === video.id
                        ? "Retrying"
                        : "Retry"}
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</section>
