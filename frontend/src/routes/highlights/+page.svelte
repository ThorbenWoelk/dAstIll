<script lang="ts">
  import { onMount } from "svelte";
  import { isAiAvailable, listHighlights } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import { DOCS_URL } from "$lib/app-config";
  import SectionNavigation from "$lib/components/SectionNavigation.svelte";
  import ThemePanel from "$lib/components/ThemePanel.svelte";
  import type {
    AiStatus,
    HighlightChannelGroup,
    HighlightSource,
  } from "$lib/types";
  import { buildWorkspaceViewHref } from "$lib/view-url";

  const dateFormatter = new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

  let aiStatus = $state<AiStatus | null>(null);
  let groups = $state<HighlightChannelGroup[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);
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

  function formatDate(value: string) {
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) {
      return "Unknown date";
    }
    return dateFormatter.format(parsed);
  }

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

<div class="page-shell min-h-screen px-3 py-4 max-lg:px-0 lg:px-6">
  <a
    href="#main-content"
    class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white"
  >
    Skip to Main Content
  </a>

  <header
    class="mx-auto flex w-full max-w-[1440px] min-w-0 flex-wrap items-start gap-3 px-4 pb-2 sm:px-2 lg:items-center"
  >
    <div class="flex min-w-0 flex-1 items-center gap-3">
      <a
        href="/"
        class="text-xl sm:text-2xl font-bold tracking-tighter text-[var(--foreground)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
        aria-label="Go to dAstIll home"
      >
        DASTILL
      </a>
      {#if aiIndicator}
        <AiStatusIndicator
          detail={aiIndicator.detail}
          dotClass={aiIndicator.dotClass}
          title={aiIndicator.title}
        />
      {/if}
    </div>

    <div class="ml-auto flex shrink-0 items-center gap-2">
      <ThemePanel />
      <SectionNavigation currentSection="highlights" docsUrl={DOCS_URL} />
    </div>
  </header>

  <main
    id="main-content"
    class="mx-auto mt-4 w-full max-w-[1120px] px-4 pb-28 sm:px-2"
  >
    <section
      class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface-frost)] p-6 shadow-sm"
    >
      <div class="flex flex-wrap items-center justify-between gap-3">
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

      {#if loading}
        <div class="mt-8 space-y-4" role="status" aria-live="polite">
          {#each Array.from({ length: 3 }) as _, index (index)}
            <div
              class="animate-pulse rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--muted)]/20 p-5"
            >
              <div class="h-4 w-40 rounded-full bg-[var(--muted)]/60"></div>
              <div
                class="mt-4 h-3 w-3/4 rounded-full bg-[var(--muted)]/50"
              ></div>
              <div
                class="mt-2 h-3 w-1/2 rounded-full bg-[var(--muted)]/40"
              ></div>
            </div>
          {/each}
        </div>
      {:else if errorMessage}
        <div
          class="mt-8 rounded-[var(--radius-md)] border border-[var(--danger-border)] bg-[var(--danger-soft)] px-4 py-3 text-[14px] text-[var(--danger-foreground)]"
        >
          {errorMessage}
        </div>
      {:else if groups.length === 0}
        <div
          class="mt-8 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--muted)]/20 px-4 py-5 text-[14px] text-[var(--soft-foreground)] opacity-70"
        >
          No highlights saved yet. Select text in a transcript or summary to
          start building your library.
        </div>
      {:else}
        <div class="mt-8 space-y-8">
          {#each groups as group}
            <section
              class="rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--background)]/90 p-5"
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
                    class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface-frost-strong)] p-4 shadow-sm"
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
                              Released {formatDate(video.published_at)}
                            </p>
                          </div>
                          <a
                            href={buildVideoHref(
                              group.channel_id,
                              video.video_id,
                              "highlights",
                            )}
                            class="inline-flex shrink-0 rounded-full border border-[var(--border)] px-3 py-1.5 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
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
                              class="block rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-[var(--muted)]/10 px-4 py-3 transition-colors hover:border-[var(--accent)]/35 hover:bg-[var(--accent-soft)]/20"
                            >
                              <div
                                class="flex flex-wrap items-center justify-between gap-2"
                              >
                                <span
                                  class="inline-flex rounded-full bg-[var(--accent-soft)]/60 px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)]"
                                >
                                  {highlight.source}
                                </span>
                                <span
                                  class="text-[11px] text-[var(--soft-foreground)] opacity-50"
                                >
                                  Saved {formatDate(highlight.created_at)}
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
    </section>
  </main>
</div>
