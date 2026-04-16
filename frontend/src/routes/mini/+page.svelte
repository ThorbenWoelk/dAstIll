<script lang="ts">
  import { goto } from "$app/navigation";
  import { authState } from "$lib/auth-state.svelte";
  import MiniArticle from "$lib/components/mini/MiniArticle.svelte";
  import MiniBottomBar from "$lib/components/mini/MiniBottomBar.svelte";
  import MiniChannelSheet from "$lib/components/mini/MiniChannelSheet.svelte";
  import MiniEmptyState from "$lib/components/mini/MiniEmptyState.svelte";
  import MiniSummaryStrip from "$lib/components/mini/MiniSummaryStrip.svelte";
  import MiniTopBar from "$lib/components/mini/MiniTopBar.svelte";
  import { createMiniKeydownHandler } from "$lib/mini/mini-keyboard";
  import { createMiniReaderState } from "$lib/mini/mini-reader-state.svelte";
  import { swipeNavigation } from "$lib/mini/use-swipe-navigation";

  const mini = createMiniReaderState();
  let channelSheetOpen = $state(false);
  let scrollContainer = $state<HTMLElement | null>(null);
  let authResolved = $state(false);

  const handleKeydown = createMiniKeydownHandler(() => ({
    stepSummary: (d) => mini.stepSummary(d),
    markActiveSummaryRead: () => mini.markActiveSummaryRead(),
    activeSummary: mini.activeSummary,
    channelSheetOpen,
    closeChannelSheet: () => {
      channelSheetOpen = false;
    },
  }));

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    mini.updateReadProgress(scrollTop, scrollHeight, clientHeight);
  }

  function resetScroll() {
    scrollContainer?.scrollTo({ top: 0, behavior: "instant" });
  }

  function stepAndScroll(delta: -1 | 1) {
    mini.stepSummary(delta);
    resetScroll();
  }

  function jumpAndScroll(videoId: string) {
    mini.jumpToSummary(videoId);
    resetScroll();
  }

  async function handleMarkRead() {
    await mini.markActiveSummaryRead();
    resetScroll();
  }

  async function handleChannelSelect(channelId: string) {
    await mini.selectChannel(channelId);
    resetScroll();
  }

  $effect(() => {
    if (!mini.reader) return;
    mini.reconcileActiveVideo();
  });

  $effect(() => {
    if (!authState.ready || authResolved) return;
    authResolved = true;
    if (authState.current.authState !== "authenticated") {
      void goto("/login?redirectTo=%2Fmini");
      return;
    }
    void mini.loadReader();
  });

  function emptyVariant(): "no-subscriptions" | "all-read" | "no-summaries" {
    if (
      mini.showUnreadOnly &&
      mini.reader?.summaries.length &&
      mini.reader.summaries.length > 0
    ) {
      return "all-read";
    }
    return "no-summaries";
  }
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
    readProgress={mini.readProgress}
    activeIndex={mini.activeIndex}
    totalCount={mini.visibleSummaries.length}
    showCounter={!!mini.activeSummary}
    showUnreadOnly={mini.showUnreadOnly}
    activeFilterCount={mini.activeFilterCount}
    unreadCount={mini.unreadCount}
    onToggleFilter={() => mini.toggleUnreadFilter()}
  />

  {#if mini.loading}
    <div class="mini-content">
      <MiniEmptyState variant="loading" />
    </div>
  {:else if mini.error}
    <div class="mini-content">
      <MiniEmptyState
        variant="error"
        errorMessage={mini.error}
        onRetry={() => mini.loadReader(mini.selectedChannelId)}
      />
    </div>
  {:else if !mini.reader || mini.reader.channels.length === 0}
    <div class="mini-content">
      <MiniEmptyState variant="no-subscriptions" />
    </div>
  {:else if !mini.activeSummary}
    <div class="mini-content">
      <MiniEmptyState
        variant={emptyVariant()}
        onClearFilter={() => mini.clearUnreadFilter()}
      />
    </div>
  {:else}
    <MiniSummaryStrip
      summaries={mini.visibleSummaries}
      activeVideoId={mini.activeVideoId}
      onSelect={jumpAndScroll}
    />

    <div
      class="mini-content"
      bind:this={scrollContainer}
      onscroll={handleScroll}
      use:swipeNavigation={{
        onSwipeLeft: () => stepAndScroll(1),
        onSwipeRight: () => stepAndScroll(-1),
        enabled: !channelSheetOpen,
      }}
    >
      <MiniArticle
        summary={mini.activeSummary}
        summaryHtml={mini.activeSummaryHtml}
        markingRead={mini.markingRead}
        canGoPrev={mini.canGoPrev}
        canGoNext={mini.canGoNext}
        contentKey={mini.contentKey}
        onMarkRead={handleMarkRead}
        onPrev={() => stepAndScroll(-1)}
        onNext={() => stepAndScroll(1)}
      />
    </div>
  {/if}

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
      onPrev={() => stepAndScroll(-1)}
      onNext={() => stepAndScroll(1)}
      onOpenChannelPicker={() => {
        channelSheetOpen = true;
      }}
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
</div>

<style>
  .mini-shell {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    background: var(--background);
    color: var(--foreground);
    overflow: hidden;
    position: relative;
  }

  .mini-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior-y: contain;
    scroll-behavior: smooth;
  }
</style>
