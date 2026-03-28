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
  import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";

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

    return createAiStatusPoller({
      intervalMs: 30000,
      onStatus: (payload) => {
        aiStatus = payload.status;
      },
    });
  });

  $effect(() => {
    mobileBottomBar.set({ kind: "hidden" });
    return () => {
      mobileBottomBar.set({ kind: "sections" });
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
      class="flex h-10 shrink-0 items-center justify-between border-b border-[var(--border-soft)]/50 px-3 sm:px-6 lg:h-12 lg:px-0"
    >
      <span
        class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-55"
      >
        Highlights
      </span>
      {#if !loading}
        <span
          class="text-[11px] font-medium text-[var(--soft-foreground)] opacity-40"
        >
          {totalHighlights} saved
        </span>
      {/if}
    </div>

    <div
      class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-3 py-3 sm:px-6 lg:px-0 lg:py-4 lg:pr-4 lg:pb-0"
    >
      {#if loading}
        <div class="space-y-3 lg:space-y-4" role="status" aria-live="polite">
          {#each Array.from({ length: 5 }) as _, index (index)}
            <div
              class="animate-pulse rounded-[var(--radius-sm)] bg-[var(--muted)]/40 p-3 lg:rounded-[var(--radius-md)] lg:border lg:border-[var(--accent-border-soft)] lg:bg-[var(--accent-wash)] lg:p-5"
            >
              <div
                class="h-3 w-32 rounded-full bg-[var(--border)] opacity-80 lg:h-4 lg:w-40"
              ></div>
              <div
                class="mt-3 h-2.5 w-11/12 rounded-full bg-[var(--border)] opacity-70 lg:mt-4 lg:h-3"
              ></div>
              <div
                class="mt-2 h-2.5 w-2/3 rounded-full bg-[var(--border)] opacity-55 lg:h-3"
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
        <div class="space-y-6 lg:space-y-8">
          {#each groups as group}
            <section
              class="border-b border-[var(--border-soft)]/50 pb-6 last:border-b-0 last:pb-0 lg:rounded-[var(--radius-lg)] lg:border lg:border-[var(--accent-border-soft)] lg:bg-[var(--panel-surface)] lg:p-5 lg:pb-5"
            >
              <div class="flex min-w-0 items-center gap-2.5 lg:gap-3">
                <div
                  class="h-7 w-7 shrink-0 overflow-hidden rounded-full bg-[var(--muted)] lg:h-10 lg:w-10"
                >
                  <img
                    src={group.channel_thumbnail_url || defaultChannelIcon}
                    alt={group.channel_name}
                    class="h-full w-full object-cover"
                    loading="lazy"
                    referrerpolicy="no-referrer"
                  />
                </div>
                <div class="min-w-0">
                  <p
                    class="truncate text-[14px] font-semibold leading-tight text-[var(--foreground)] lg:text-[17px]"
                  >
                    {group.channel_name}
                  </p>
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 lg:text-[12px] lg:font-medium lg:normal-case lg:tracking-normal lg:opacity-55"
                  >
                    {group.videos.length} video{group.videos.length === 1
                      ? ""
                      : "s"}
                  </p>
                </div>
              </div>

              <div class="mt-4 space-y-5 lg:mt-5 lg:space-y-4">
                {#each group.videos as video}
                  <article
                    class="border-b border-[var(--border-soft)]/35 pb-5 last:border-b-0 last:pb-0 lg:rounded-[var(--radius-md)] lg:border lg:border-[var(--accent-border-soft)] lg:bg-[var(--panel-surface-strong)] lg:p-4 lg:shadow-sm lg:last:border-[var(--accent-border-soft)]"
                  >
                    <div
                      class="flex flex-col gap-3 lg:flex-row lg:items-start lg:gap-4"
                    >
                      <div
                        class="hidden w-[200px] shrink-0 overflow-hidden rounded-[var(--radius-sm)] bg-[var(--muted)] lg:block xl:w-[220px]"
                      >
                        {#if video.thumbnail_url}
                          <div class="aspect-video w-full">
                            <img
                              src={video.thumbnail_url}
                              alt={video.title}
                              class="h-full w-full object-cover"
                              loading="lazy"
                              referrerpolicy="no-referrer"
                            />
                          </div>
                        {:else}
                          <div
                            class="aspect-video w-full bg-[var(--muted)]"
                          ></div>
                        {/if}
                      </div>

                      <div class="min-w-0 flex-1">
                        <div
                          class="flex flex-wrap items-start justify-between gap-2 gap-y-1"
                        >
                          <div class="min-w-0 flex-1">
                            <h2
                              class="line-clamp-2 text-[14px] font-semibold leading-snug text-[var(--foreground)] lg:line-clamp-none lg:text-[17px] lg:leading-tight"
                            >
                              {video.title}
                            </h2>
                            <p
                              class="mt-0.5 text-[11px] text-[var(--soft-foreground)] opacity-55"
                            >
                              {formatShortDate(video.published_at)}
                            </p>
                          </div>
                          <a
                            href={buildVideoHref(
                              group.channel_id,
                              video.video_id,
                              "highlights",
                            )}
                            class="inline-flex shrink-0 rounded-full px-2 py-1 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--accent-strong)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] lg:border lg:border-[var(--accent-border-soft)] lg:px-3 lg:py-1.5 lg:text-[var(--foreground)] lg:hover:border-[var(--accent)] lg:hover:text-[var(--accent)]"
                          >
                            Open
                          </a>
                        </div>

                        <div class="mt-3 space-y-2 lg:mt-4 lg:space-y-3">
                          {#each video.highlights as highlight (highlight.id)}
                            {@const hid = Number(highlight.id)}
                            <div
                              class="relative rounded-[var(--radius-sm)] border-l-2 border-[var(--accent)]/40 bg-[var(--accent-wash)]/60 pl-3 pr-16 py-2.5 transition-colors hover:bg-[var(--accent-wash)] lg:border lg:border-[var(--accent-border-soft)] lg:border-l-[var(--accent-border-soft)] lg:bg-[var(--accent-wash)] lg:pl-4 lg:pr-[4.5rem] lg:py-3 lg:hover:border-[var(--accent)]/35"
                            >
                              <a
                                href={buildVideoHref(
                                  group.channel_id,
                                  video.video_id,
                                  highlight.source,
                                )}
                                class="block text-[var(--foreground)]"
                              >
                                <div
                                  class="flex flex-wrap items-baseline justify-between gap-x-2 gap-y-0.5"
                                >
                                  <span
                                    class="text-[9px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)] lg:inline-flex lg:rounded-full lg:bg-[var(--accent-wash-strong)] lg:px-2 lg:py-1 lg:text-[10px]"
                                  >
                                    {highlight.source}
                                  </span>
                                  <span
                                    class="text-[10px] text-[var(--soft-foreground)] opacity-50 lg:text-[11px]"
                                  >
                                    {formatShortDate(highlight.created_at)}
                                  </span>
                                </div>
                                <p
                                  class="mt-2 line-clamp-6 whitespace-pre-wrap text-[13px] leading-snug lg:line-clamp-none lg:mt-3 lg:text-[15px] lg:leading-relaxed"
                                >
                                  {highlight.text}
                                </p>
                              </a>
                              <div
                                class="absolute right-1 top-1.5 flex items-center gap-0.5 lg:right-2 lg:top-2"
                              >
                                <button
                                  type="button"
                                  class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash-strong)] hover:text-[var(--foreground)] lg:h-8 lg:w-8"
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
                                      size={16}
                                      strokeWidth={2}
                                      className="text-[var(--accent)]"
                                    />
                                  {:else}
                                    <CopyIcon size={14} strokeWidth={2} />
                                  {/if}
                                </button>
                                <button
                                  type="button"
                                  class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash-strong)] hover:text-[var(--danger)] disabled:cursor-not-allowed disabled:opacity-50 lg:h-8 lg:w-8"
                                  disabled={deletingHighlightId === hid}
                                  onclick={() => void removeHighlightEntry(hid)}
                                  aria-label="Delete highlight"
                                >
                                  <TrashIcon
                                    size={14}
                                    strokeWidth={2.2}
                                    class={deletingHighlightId === hid
                                      ? "animate-pulse"
                                      : ""}
                                  />
                                </button>
                              </div>
                            </div>
                          {/each}
                        </div>
                      </div>
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
