<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { getPreferences, isAiAvailable } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import { mobileBottomBar } from "$lib/mobile-navigation/mobileBottomBar";
  import type { AiStatus, VocabularyReplacement } from "$lib/types";
  import { createAiStatusPoller } from "$lib/utils/ai-poller";
  import { formatVocabularyAddedAt } from "$lib/vocabulary";

  let aiStatus = $state<AiStatus | null>(null);
  let vocabulary = $state<VocabularyReplacement[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);

  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );
  let sortedVocabulary = $derived(
    vocabulary.toSorted(
      (a, b) => new Date(b.added_at).getTime() - new Date(a.added_at).getTime(),
    ),
  );

  function openGuide() {
    void goto("/?guide=0");
  }

  async function loadPage() {
    loading = true;
    errorMessage = null;

    try {
      const [preferences, aiHealth] = await Promise.all([
        getPreferences(),
        isAiAvailable(),
      ]);
      vocabulary = preferences.vocabulary_replacements ?? [];
      aiStatus = aiHealth.status;
    } catch (error) {
      errorMessage = (error as Error).message;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
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
  currentSection="vocabulary"
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
        Vocabulary
      </span>
      {#if !loading}
        <span
          class="text-[11px] font-medium text-[var(--soft-foreground)] opacity-40"
        >
          {vocabulary.length} saved
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
              class="animate-pulse rounded-[var(--radius-sm)] bg-[var(--muted)]/40 p-4 lg:rounded-[var(--radius-md)] lg:bg-[var(--panel-surface)]"
            >
              <div class="h-3 w-28 rounded-full bg-[var(--border)]/80"></div>
              <div
                class="mt-4 h-4 w-3/5 rounded-full bg-[var(--border)]/60"
              ></div>
              <div
                class="mt-2 h-3 w-2/5 rounded-full bg-[var(--border)]/45"
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
      {:else if sortedVocabulary.length === 0}
        <div class="max-w-2xl space-y-3 px-0.5">
          <p
            class="text-[13px] leading-relaxed text-[var(--soft-foreground)] opacity-60 lg:text-[14px]"
          >
            No vocabulary saved yet. Select a misspelled name, place, company,
            or term in a transcript or summary, then choose <strong
              class="text-[var(--foreground)] opacity-90">Correct</strong
            > to store the canonical spelling for future summaries.
          </p>
        </div>
      {:else}
        <div class="grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
          {#each sortedVocabulary as entry (`${entry.from}-${entry.to}-${entry.added_at}`)}
            <article
              class="rounded-[var(--radius-md)] bg-[var(--panel-surface)] p-4 shadow-sm"
            >
              <div class="flex items-start justify-between gap-3">
                <span
                  class="rounded-full bg-[var(--accent-wash)] px-2.5 py-1 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--accent-strong)]"
                >
                  Added {formatVocabularyAddedAt(entry.added_at)}
                </span>
              </div>
              <div class="mt-4 space-y-3">
                <div>
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-50"
                  >
                    Replace
                  </p>
                  <p
                    class="mt-1 text-[15px] font-medium text-[var(--foreground)]"
                  >
                    {entry.from}
                  </p>
                </div>
                <div class="flex items-center gap-2 text-[var(--accent)]">
                  <span class="h-px flex-1 bg-[var(--accent)]/20"></span>
                  <span
                    class="text-[11px] font-bold uppercase tracking-[0.12em]"
                    >to</span
                  >
                  <span class="h-px flex-1 bg-[var(--accent)]/20"></span>
                </div>
                <div>
                  <p
                    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] opacity-50"
                  >
                    Use
                  </p>
                  <p
                    class="mt-1 text-[15px] font-semibold text-[var(--accent-strong)]"
                  >
                    {entry.to}
                  </p>
                </div>
              </div>
            </article>
          {/each}
        </div>
      {/if}
    </div>
  </section>

  {#if errorMessage}
    <ErrorToast
      message={errorMessage}
      onDismiss={() => (errorMessage = null)}
    />
  {/if}
</WorkspaceShell>
