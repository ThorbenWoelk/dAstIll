<script lang="ts">
  import { goto } from "$app/navigation";
  import { authState } from "$lib/auth/state.svelte";
  import MiniArticle from "$lib/components/mini/MiniArticle.svelte";
  import MiniBottomBar from "$lib/components/mini/MiniBottomBar.svelte";
  import MiniChannelSheet from "$lib/components/mini/MiniChannelSheet.svelte";
  import MiniEmptyState from "$lib/components/mini/MiniEmptyState.svelte";
  import MiniSummaryStrip from "$lib/components/mini/MiniSummaryStrip.svelte";
  import MiniTopBar from "$lib/components/mini/MiniTopBar.svelte";
  import ErrorToast from "$lib/components/ErrorToast.svelte";
  import VocabularyReplacementModal from "$lib/components/VocabularyReplacementModal.svelte";
  import { getAuthStorageScopeKey } from "$lib/auth/storage";
  import {
    shouldRedirectMiniToLogin,
    shouldReloadMiniForAuthScope,
  } from "$lib/mini/mini-auth-scope";
  import { createMiniKeydownHandler } from "$lib/mini/mini-keyboard";
  import { createMiniReaderState } from "$lib/mini/mini-reader-state.svelte";
  import { createMiniScrollController } from "$lib/mini/mini-scroll.svelte";
  import { pullRefresh } from "$lib/mini/use-pull-refresh";
  import { swipeNavigation } from "$lib/mini/use-swipe-navigation";

  const mini = createMiniReaderState();
  const scroll = createMiniScrollController(mini);
  let channelSheetOpen = $state(false);
  let loadedAuthScopeKey = $state<string | null>(null);
  let loadingAuthScopeKey = $state<string | null>(null);
  let scrollContainer = $state<HTMLElement | null>(null);
  const authScopeKey = $derived(getAuthStorageScopeKey(authState.current));

  $effect(() => {
    scroll.bind(scrollContainer);
  });

  const handleKeydown = createMiniKeydownHandler(() => ({
    stepSummary: stepAndScroll,
    markActiveSummaryRead: handleMarkRead,
    activeSummary: mini.activeSummary,
    channelSheetOpen,
    closeChannelSheet: () => {
      channelSheetOpen = false;
    },
  }));

  function stepAndScroll(delta: -1 | 1) {
    mini.stepSummary(delta);
    scroll.reset();
  }

  function jumpAndScroll(videoId: string) {
    mini.jumpToSummary(videoId);
    scroll.reset();
  }

  async function handleMarkRead() {
    await mini.markActiveSummaryRead();
    scroll.reset();
  }

  async function handleMarkReadAndAdvance() {
    await mini.markActiveSummaryReadAndAdvance();
    scroll.reset();
  }

  async function handleChannelSelect(channelId: string) {
    await mini.selectChannel(channelId);
    scroll.reset();
  }

  async function refreshAndReset() {
    await mini.refreshReader();
    scroll.reset();
  }

  $effect(() => {
    if (!mini.reader) return;
    mini.reconcileActiveVideo();
  });

  $effect(() => {
    mini.activeSummary?.video_id;
    mini.hydrateActiveSummaryHighlights();
  });

  function reloadMiniForAuthScope(nextAuthScopeKey: string) {
    loadingAuthScopeKey = nextAuthScopeKey;
    if (
      loadedAuthScopeKey !== null ||
      mini.reader !== null ||
      mini.preferencesLoaded
    ) {
      mini.resetForAuthScopeChange();
    }
    void (async () => {
      try {
        await Promise.all([
          mini.loadPreferences(),
          mini.loadReader(undefined, undefined, { bypassCache: true }),
        ]);
        if (getAuthStorageScopeKey(authState.current) !== nextAuthScopeKey) {
          return;
        }
        loadedAuthScopeKey = nextAuthScopeKey;
      } finally {
        if (loadingAuthScopeKey === nextAuthScopeKey) {
          loadingAuthScopeKey = null;
        }
      }
    })();
  }

  $effect(() => {
    if (!authState.ready) return;

    if (shouldRedirectMiniToLogin(authState.current.authState)) {
      if (
        loadedAuthScopeKey !== null ||
        loadingAuthScopeKey !== null ||
        mini.reader !== null ||
        mini.preferencesLoaded
      ) {
        mini.resetForAuthScopeChange();
      }
      if (loadedAuthScopeKey !== null) {
        loadedAuthScopeKey = null;
      }
      if (loadingAuthScopeKey !== null) {
        loadingAuthScopeKey = null;
      }
      void goto("/login?redirectTo=%2Fmini");
      return;
    }

    if (
      !shouldReloadMiniForAuthScope({
        authReady: authState.ready,
        loadedAuthScopeKey,
        loadingAuthScopeKey,
        authScopeKey,
      })
    ) {
      return;
    }

    reloadMiniForAuthScope(authScopeKey);
  });
</script>

<svelte:head>
  <title>dastill-mini</title>
  <meta
    name="description"
    content="A minimal summary reader for your subscribed dAstIll channels."
  />
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="mini-shell">
  <MiniTopBar
    activeIndex={mini.activeIndex}
    totalCount={mini.visibleSummaries.length}
    showCounter={!!mini.activeSummary}
    showUnreadOnly={mini.showUnreadOnly}
    activeFilterCount={mini.activeFilterCount}
    unreadCount={mini.unreadCount}
    onToggleFilter={() => mini.toggleUnreadFilter()}
    channelName={mini.activeSummary?.channel_name ??
      mini.reader?.channels.find((c) => c.id === mini.selectedChannelId)
        ?.name ??
      null}
    canPickChannel={!!mini.reader && mini.reader.channels.length > 0}
    onOpenChannelPicker={() => {
      channelSheetOpen = true;
    }}
  />

  <div class="mini-main">
    {#if mini.status === "loading" && !mini.activeSummary}
      <div class="mini-article-pane">
        <MiniEmptyState variant="loading" />
      </div>
    {:else if mini.status === "error"}
      <div
        class="mini-article-pane"
        use:pullRefresh={{
          onRefresh: refreshAndReset,
          enabled: !channelSheetOpen,
        }}
      >
        <MiniEmptyState
          variant="error"
          errorMessage={mini.errorMessage ?? undefined}
          onRetry={() => mini.loadReader(mini.selectedChannelId)}
        />
      </div>
    {:else if mini.status === "empty"}
      <div
        class="mini-article-pane"
        use:pullRefresh={{
          onRefresh: refreshAndReset,
          enabled: !channelSheetOpen,
        }}
      >
        <MiniEmptyState
          variant={mini.emptyVariant}
          onClearFilter={() => mini.clearUnreadFilter()}
        />
      </div>
    {:else}
      <MiniSummaryStrip
        summaries={mini.visibleSummaries}
        activeVideoId={mini.activeVideoId}
        collapsed={scroll.scrolledFromTop}
        onSelect={jumpAndScroll}
      />

      <div
        class="mini-article-pane"
        bind:this={scrollContainer}
        onscroll={() => scroll.onScroll()}
        use:pullRefresh={{
          onRefresh: refreshAndReset,
          enabled: !channelSheetOpen,
        }}
        use:swipeNavigation={{
          onSwipeLeft: () => stepAndScroll(1),
          onSwipeRight: () => stepAndScroll(-1),
          enabled: !channelSheetOpen,
        }}
      >
        {#if mini.status === "loading"}
          <span class="stale-badge" aria-live="polite">Updating…</span>
        {/if}
        <MiniArticle
          summary={mini.activeSummary}
          summaryHtml={mini.activeSummaryHtml}
          contentKey={mini.contentKey}
          highlights={mini.activeSummaryHighlights}
          creatingHighlight={mini.creatingHighlight &&
            mini.creatingHighlightVideoId === mini.activeSummary.video_id}
          creatingVocabularyReplacement={mini.creatingVocabularyReplacement}
          deletingHighlightId={mini.deletingHighlightId}
          onCreateHighlight={(payload) => mini.saveSelectionHighlight(payload)}
          onCreateVocabularyReplacement={(selectedText) =>
            mini.openVocabularyReplacement(selectedText)}
          onDeleteHighlight={(highlightId) =>
            mini.deleteExistingHighlight(highlightId)}
        />
      </div>
    {/if}
  </div>

  {#if mini.reader && mini.reader.channels.length > 0}
    <MiniBottomBar
      channelName={mini.activeSummary?.channel_name ??
        mini.reader.channels.find((c) => c.id === mini.selectedChannelId)
          ?.name ??
        null}
      canGoPrev={mini.canGoPrev}
      canGoNext={mini.canGoNext}
      activeIndex={mini.activeIndex}
      totalCount={mini.visibleSummaries.length}
      showReadCheckbox={scroll.scrolledFromTop && !!mini.activeSummary}
      activeSummaryRead={mini.activeSummary?.read ?? false}
      markingRead={mini.markingRead}
      onPrev={() => stepAndScroll(-1)}
      onNext={() => stepAndScroll(1)}
      onOpenChannelPicker={() => {
        channelSheetOpen = true;
      }}
      onMarkReadAndAdvance={handleMarkReadAndAdvance}
    />

    <MiniChannelSheet
      open={channelSheetOpen}
      channels={mini.reader.channels}
      selectedChannelId={mini.selectedChannelId}
      summaries={mini.reader.summaries}
      onSelect={handleChannelSelect}
      onClose={() => {
        channelSheetOpen = false;
      }}
    />
  {/if}

  <VocabularyReplacementModal
    show={Boolean(mini.vocabularyModalSource)}
    source={mini.vocabularyModalSource ?? ""}
    value={mini.vocabularyModalValue}
    busy={mini.creatingVocabularyReplacement}
    onValueChange={(value) => mini.setVocabularyModalValue(value)}
    onConfirm={() => void mini.confirmVocabularyReplacement()}
    onCancel={() => mini.closeVocabularyModal()}
  />

  {#if mini.error}
    <ErrorToast
      message={mini.error}
      onDismiss={() => mini.clearActionError()}
    />
  {/if}
</div>

<style>
  .mini-shell {
    /* Mini is monochrome. Neutralize every palette-mixed token so
       data-color / theme swaps never leak into this surface. */
    --accent: var(--foreground);
    --accent-strong: var(--foreground);
    --accent-soft: color-mix(in srgb, var(--foreground) 8%, var(--surface));
    --accent-wash: color-mix(in srgb, var(--foreground) 6%, var(--surface));
    --accent-wash-strong: color-mix(
      in srgb,
      var(--foreground) 12%,
      var(--surface)
    );
    --muted: color-mix(in srgb, var(--foreground) 7%, var(--background));
    --border: color-mix(in srgb, var(--foreground) 18%, var(--background));
    --border-soft: color-mix(in srgb, var(--foreground) 9%, var(--background));

    display: flex;
    flex-direction: column;
    height: 100dvh;
    background: var(--background);
    color: var(--foreground);
    overflow: hidden;
    position: relative;
  }

  .mini-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .mini-article-pane {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior-y: contain;
    scroll-behavior: smooth;
    position: relative;
  }

  .stale-badge {
    position: absolute;
    top: var(--space-md);
    right: var(--space-md);
    z-index: 10;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--soft-foreground);
    opacity: 0.6;
  }

  @media (min-width: 960px) {
    .mini-main {
      flex-direction: row;
    }
  }
</style>
