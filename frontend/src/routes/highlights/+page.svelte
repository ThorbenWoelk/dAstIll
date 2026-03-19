<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { isAiAvailable, listHighlights } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import { DOCS_URL } from "$lib/app-config";
  import FeatureGuide, {
    type TourStep,
  } from "$lib/components/FeatureGuide.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import type {
    AiStatus,
    HighlightChannelGroup,
    HighlightSource,
    SearchResult,
  } from "$lib/types";
  import { buildWorkspaceViewHref } from "$lib/view-url";
  import { formatShortDate } from "$lib/utils/date";

  let aiStatus = $state<AiStatus | null>(null);
  let groups = $state<HighlightChannelGroup[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);

  let guideOpen = $state(false);
  let guideStep = $state(0);
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  const tourSteps: TourStep[] = [
    {
      selector: "nav[aria-label='Workspace sections']",
      title: "App navigation",
      body: "Jump between workspace, queue, highlights, and docs from the same shared header.",
      placement: "bottom",
    },
    {
      selector: "#main-content",
      title: "Highlights library",
      body: "Review saved passages grouped by channel and video, then jump straight back into the workspace for context.",
      placement: "top",
    },
  ];
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
    guideStep = 0;
    guideOpen = true;
  }

  function closeGuide() {
    guideOpen = false;
  }

  function setGuideStep(step: number) {
    guideStep = step;
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
      errorMessage = (error as Error).message;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadPage();

    const timer = setInterval(() => {
      void isAiAvailable()
        .then((status) => {
          aiStatus = status.status;
        })
        .catch(() => {
          aiStatus = "offline";
        });
    }, 30000);

    return () => clearInterval(timer);
  });
</script>

<WorkspaceShell
  currentSection="highlights"
  {aiIndicator}
  onOpenGuide={openGuide}
>
  <section
    id="content-view"
    class="fade-in stagger-3 relative z-10 flex h-full min-h-0 min-w-0 flex-col overflow-visible lg:gap-4 lg:px-8 lg:pt-6 lg:pb-6"
  >
    <div
      class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--accent-border-soft)] px-4 py-4 sm:px-6 lg:px-0"
    >
      <div>
        <p
          class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-50"
        >
          Library
        </p>
        <h1
          class="mt-2 font-serif text-[32px] font-bold tracking-tight text-[var(--foreground)]"
        >
          Highlights
        </h1>
        <p class="mt-2 text-[13px] text-[var(--soft-foreground)]">
          Saved passages across your indexed transcripts and summaries.
        </p>
      </div>

      <div class="text-right">
        <p
          class="text-[11px] font-bold uppercase tracking-[0.14em] text-[var(--soft-foreground)] opacity-50"
        >
          Saved passages
        </p>
        <p
          class="mt-2 text-[28px] font-bold tracking-tight text-[var(--foreground)]"
        >
          {totalHighlights}
        </p>
      </div>
    </div>

    <div
      class="custom-scrollbar mobile-bottom-stack-padding w-full min-h-0 flex-1 overflow-y-auto px-4 py-4 sm:px-6 lg:px-0 lg:pr-4 lg:pb-0"
    >
      {#if loading}
        <div class="space-y-4" role="status" aria-live="polite">
          {#each Array.from({ length: 5 }) as _, index (index)}
            <div
              class="animate-pulse rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--accent-wash)] p-5"
            >
              <div
                class="h-4 w-40 rounded-full bg-[var(--border)] opacity-80"
              ></div>
              <div
                class="mt-4 h-3 w-3/4 rounded-full bg-[var(--border)] opacity-70"
              ></div>
              <div
                class="mt-2 h-3 w-1/2 rounded-full bg-[var(--border)] opacity-55"
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
        <p class="px-1 text-[14px] text-[var(--soft-foreground)] opacity-60">
          No highlights saved yet. Select text in a transcript or summary to
          start building your library.
        </p>
      {:else}
        <div class="space-y-8">
          {#each groups as group}
            <section
              class="rounded-[var(--radius-lg)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface)] p-5"
            >
              <div class="flex flex-wrap items-center justify-between gap-3">
                <div class="flex min-w-0 items-center gap-3">
                  <div
                    class="h-11 w-11 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
                  >
                    {#if group.channel_thumbnail_url}
                      <img
                        src={group.channel_thumbnail_url}
                        alt={group.channel_name}
                        class="h-full w-full object-cover"
                      />
                    {/if}
                  </div>
                  <div class="min-w-0">
                    <p
                      class="truncate text-[18px] font-semibold text-[var(--foreground)]"
                    >
                      {group.channel_name}
                    </p>
                    <p
                      class="text-[12px] text-[var(--soft-foreground)] opacity-55"
                    >
                      {group.videos.length} video{group.videos.length === 1
                        ? ""
                        : "s"}
                    </p>
                  </div>
                </div>
              </div>

              <div class="mt-5 space-y-4">
                {#each group.videos as video}
                  <article
                    class="rounded-[var(--radius-md)] border border-[var(--accent-border-soft)] bg-[var(--panel-surface-strong)] p-4 shadow-sm"
                  >
                    <div class="flex flex-col gap-4 sm:flex-row sm:items-start">
                      <div
                        class="aspect-video w-full overflow-hidden rounded-[var(--radius-sm)] bg-[var(--muted)] sm:w-[220px]"
                      >
                        {#if video.thumbnail_url}
                          <img
                            src={video.thumbnail_url}
                            alt={video.title}
                            class="h-full w-full object-cover"
                          />
                        {/if}
                      </div>

                      <div class="min-w-0 flex-1">
                        <div
                          class="flex flex-wrap items-start justify-between gap-3"
                        >
                          <div class="min-w-0">
                            <h2
                              class="text-[18px] font-semibold leading-tight text-[var(--foreground)]"
                            >
                              {video.title}
                            </h2>
                            <p
                              class="mt-1 text-[12px] text-[var(--soft-foreground)] opacity-55"
                            >
                              Released {formatShortDate(video.published_at)}
                            </p>
                          </div>
                          <a
                            href={buildVideoHref(
                              group.channel_id,
                              video.video_id,
                              "highlights",
                            )}
                            class="inline-flex shrink-0 rounded-full border border-[var(--accent-border-soft)] px-3 py-1.5 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:bg-[var(--accent-wash)] hover:text-[var(--accent)]"
                          >
                            Open video
                          </a>
                        </div>

                        <div class="mt-4 space-y-3">
                          {#each video.highlights as highlight (highlight.id)}
                            <a
                              href={buildVideoHref(
                                group.channel_id,
                                video.video_id,
                                highlight.source,
                              )}
                              class="block rounded-[var(--radius-sm)] border border-[var(--accent-border-soft)] bg-[var(--accent-wash)] px-4 py-3 transition-colors hover:border-[var(--accent)]/35 hover:bg-[var(--accent-wash-strong)]"
                            >
                              <div
                                class="flex flex-wrap items-center justify-between gap-2"
                              >
                                <span
                                  class="inline-flex rounded-full bg-[var(--accent-wash-strong)] px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)]"
                                >
                                  {highlight.source}
                                </span>
                                <span
                                  class="text-[11px] text-[var(--soft-foreground)] opacity-50"
                                >
                                  Saved {formatShortDate(highlight.created_at)}
                                </span>
                              </div>
                              <p
                                class="mt-3 whitespace-pre-wrap text-[15px] leading-relaxed text-[var(--foreground)]"
                              >
                                {highlight.text}
                              </p>
                            </a>
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

  <FeatureGuide
    open={guideOpen}
    step={guideStep}
    steps={tourSteps}
    docsUrl={DOCS_URL}
    onClose={closeGuide}
    onStep={setGuideStep}
  />
</WorkspaceShell>
