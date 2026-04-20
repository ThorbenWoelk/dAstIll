<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { deleteHighlight, isAiAvailable, listHighlights } from "$lib/api";
  import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import { createAiStatusPoller } from "$lib/utils/ai-poller";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import CopyIcon from "$lib/components/icons/CopyIcon.svelte";
  import PenIcon from "$lib/components/icons/PenIcon.svelte";
  import TextSquareIcon from "$lib/components/icons/TextSquareIcon.svelte";
  import TrashIcon from "$lib/components/icons/TrashIcon.svelte";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import type {
    AiStatus,
    HighlightChannelGroup,
    HighlightSource,
    SearchResult,
  } from "$lib/types";
  import { buildWorkspaceViewHref } from "$lib/view-url";
  import { formatShortDate } from "$lib/utils/date";
  import { removeHighlightFromGroups } from "$lib/utils/highlights";

  let aiStatus = $state<AiStatus | null>(null);
  let groups = $state<HighlightChannelGroup[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);
  let deletingHighlightId = $state<number | null>(null);
  let deleteError = $state<string | null>(null);
  let copiedHighlightId = $state<number | null>(null);
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;

  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  const totalHighlights = $derived(
    groups.reduce(
      (sum, channel) =>
        sum +
        channel.videos.reduce(
          (channelSum, video) => channelSum + video.highlights.length,
          0,
        ),
      0,
    ),
  );

  function buildVideoHref(
    channelId: string,
    videoId: string,
    contentMode: "highlights" | HighlightSource,
  ) {
    return buildWorkspaceViewHref({
      selectedChannelId: channelId,
      selectedVideoId: videoId,
      contentMode,
      videoTypeFilter: "all",
      acknowledgedFilter: "all",
    });
  }

  function openGuide() {
    void goto("/?guide=0");
  }

  async function handleSearchResultSelect(
    result: SearchResult,
    mode: "transcript" | "summary",
  ) {
    await goto(
      buildWorkspaceViewHref({
        selectedChannelId: result.channel_id,
        selectedVideoId: result.video_id,
        contentMode: mode,
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    );
  }

  async function loadPage() {
    loading = true;
    errorMessage = null;

    try {
      const [highlightGroups, aiHealth] = await Promise.all([
        listHighlights(),
        isAiAvailable(),
      ]);
      groups = highlightGroups;
      aiStatus = aiHealth.status;
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        errorMessage = (error as Error).message;
      }
    } finally {
      loading = false;
    }
  }

  async function copyHighlightText(highlightId: number, text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedHighlightId = highlightId;
      if (copyResetTimer) clearTimeout(copyResetTimer);
      copyResetTimer = setTimeout(() => {
        copiedHighlightId = null;
        copyResetTimer = null;
      }, 2000);
    } catch {
      /* clipboard may be unavailable */
    }
  }

  async function removeHighlightEntry(highlightId: number) {
    deletingHighlightId = highlightId;
    deleteError = null;
    try {
      await deleteHighlight(highlightId);
      groups = removeHighlightFromGroups(groups, highlightId);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        deleteError = (error as Error).message;
      }
    } finally {
      deletingHighlightId = null;
    }
  }

  onMount(() => {
    const guideParam = new URL(window.location.href).searchParams.get("guide");
    if (guideParam !== null) {
      void goto(`/?guide=${guideParam}`, { replaceState: true });
      return () => {};
    }

    void loadPage();

    const stopPoller = createAiStatusPoller({
      intervalMs: 30000,
      onStatus: (payload) => {
        aiStatus = payload.status;
      },
    });

    return () => {
      if (copyResetTimer) {
        clearTimeout(copyResetTimer);
        copyResetTimer = null;
      }
      stopPoller();
    };
  });
</script>

<WorkspaceShell
  currentSection="highlights"
  {aiIndicator}
  onOpenGuide={openGuide}
>
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav />
  {/snippet}
  <section
    id="content-view"
    class="fade-in stagger-3 relative z-10 flex h-full min-h-0 min-w-0 flex-col overflow-visible lg:gap-4 lg:px-8 lg:pb-6"
  >
    <div
      class="flex h-12 shrink-0 items-center justify-between px-3 sm:px-6 lg:px-0"
    >
      <h2 class="text-lg font-semibold tracking-tight text-[var(--foreground)]">
        Highlights Archive
      </h2>
      {#if !loading}
        <span class="text-xs font-medium text-[var(--soft-foreground)]">
          {totalHighlights} saved
        </span>
      {/if}
    </div>

    <div
      class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-3 py-3 sm:px-6 lg:px-0 lg:py-4 lg:pr-4 lg:pb-0"
    >
      {#if loading}
        <div class="grid gap-4" role="status" aria-live="polite">
          {#each Array.from({ length: 5 }) as _, index (index)}
            <div
              class="animate-pulse rounded-lg border border-[var(--border)] bg-[var(--surface-strong)] p-5"
            >
              <div
                class="h-3 w-11/12 rounded-full bg-[var(--muted)] opacity-70"
              ></div>
              <div
                class="mt-2 h-3 w-2/3 rounded-full bg-[var(--muted)] opacity-60"
              ></div>
              <div
                class="mt-4 h-2.5 w-24 rounded-full bg-[var(--muted)] opacity-50"
              ></div>
            </div>
          {/each}
        </div>
      {:else if errorMessage}
        <div
          class="rounded-[var(--radius-md)] border border-[var(--danger-border)] bg-[var(--danger-soft)] px-4 py-3 text-[14px] text-[var(--danger-foreground)]"
        >
          {errorMessage}
        </div>
      {:else if groups.length === 0}
        <p
          class="px-0.5 text-[13px] leading-relaxed text-[var(--soft-foreground)] opacity-60 lg:text-[14px]"
        >
          No highlights saved yet. Select text in a transcript or summary to
          start building your library.
        </p>
      {:else}
        <div class="space-y-8 lg:space-y-10">
          {#each groups as group}
            <section>
              <div class="mb-3 flex min-w-0 items-center gap-2.5">
                <div
                  class="h-6 w-6 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
                >
                  <img
                    src={group.channel_thumbnail_url || defaultChannelIcon}
                    alt={group.channel_name}
                    class="h-full w-full object-cover"
                    loading="lazy"
                    referrerpolicy="no-referrer"
                  />
                </div>
                <h3
                  class="truncate text-xs font-semibold uppercase tracking-wider text-[var(--soft-foreground)]"
                >
                  {group.channel_name}
                </h3>
              </div>

              <div class="space-y-5">
                {#each group.videos as video}
                  <article>
                    <div
                      class="mb-3 flex items-start justify-between gap-3 px-0.5"
                    >
                      <h2
                        class="min-w-0 flex-1 truncate text-[13px] font-medium text-[var(--foreground)]"
                      >
                        {video.title}
                      </h2>
                      <a
                        href={buildVideoHref(
                          group.channel_id,
                          video.video_id,
                          "highlights",
                        )}
                        class="shrink-0 text-[11px] font-medium text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)]"
                      >
                        Open
                      </a>
                    </div>

                    <div class="grid gap-4">
                      {#each video.highlights as highlight (highlight.id)}
                        {@const hid = Number(highlight.id)}
                        <div
                          class="group relative rounded-lg border border-[var(--border)] bg-[var(--surface-strong)] p-5 transition-colors hover:border-[var(--soft-foreground)]/30"
                        >
                          <a
                            href={buildVideoHref(
                              group.channel_id,
                              video.video_id,
                              highlight.source,
                            )}
                            class="block"
                          >
                            <p
                              class="mb-4 font-serif text-sm italic leading-relaxed text-[var(--foreground)]"
                            >
                              {highlight.text}
                            </p>
                          </a>
                          <div class="flex items-center justify-between">
                            <div
                              class="flex items-center gap-1.5 text-[10px] font-medium uppercase tracking-wide text-[var(--soft-foreground)]"
                            >
                              {#if highlight.source === "summary"}
                                <PenIcon size={12} strokeWidth={1.5} />
                                AI Summary
                              {:else}
                                <TextSquareIcon size={12} strokeWidth={1.5} />
                                Transcript
                              {/if}
                              <span class="ml-1.5 opacity-60"
                                >· {formatShortDate(highlight.created_at)}</span
                              >
                            </div>
                            <div
                              class="flex items-center gap-2 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100"
                            >
                              <button
                                type="button"
                                class="inline-flex h-7 w-7 items-center justify-center rounded-md text-[var(--soft-foreground)] transition-colors hover:text-[var(--foreground)]"
                                data-tooltip={copiedHighlightId === hid
                                  ? "Copied"
                                  : "Copy"}
                                aria-label={copiedHighlightId === hid
                                  ? "Copied"
                                  : "Copy highlight"}
                                onclick={() =>
                                  void copyHighlightText(hid, highlight.text)}
                              >
                                {#if copiedHighlightId === hid}
                                  <CheckIcon
                                    size={14}
                                    strokeWidth={2}
                                    className="text-[var(--accent)]"
                                  />
                                {:else}
                                  <CopyIcon size={14} strokeWidth={1.8} />
                                {/if}
                              </button>
                              <button
                                type="button"
                                class="inline-flex h-7 w-7 items-center justify-center rounded-md text-[var(--soft-foreground)] transition-colors hover:text-[var(--danger)] disabled:cursor-not-allowed disabled:opacity-50"
                                disabled={deletingHighlightId === hid}
                                onclick={() => void removeHighlightEntry(hid)}
                                aria-label="Delete highlight"
                                data-tooltip="Delete"
                              >
                                <TrashIcon
                                  size={14}
                                  strokeWidth={1.8}
                                  class={deletingHighlightId === hid
                                    ? "animate-pulse"
                                    : ""}
                                />
                              </button>
                            </div>
                          </div>
                        </div>
                      {/each}
                    </div>
                  </article>
                {/each}
              </div>
            </section>
          {/each}
        </div>
      {/if}
    </div>
  </section>

  {#if deleteError}
    <ErrorToast
      message={deleteError}
      onDismiss={() => {
        deleteError = null;
      }}
    />
  {/if}
</WorkspaceShell>
