<script lang="ts">
  import { onMount } from "svelte";
  import AddSourceFeedbackToast from "$lib/components/AddSourceFeedbackToast.svelte";
  import VocabularyReplacementModal from "$lib/components/VocabularyReplacementModal.svelte";
  import MobileTopBarVideoFilters from "$lib/components/mobile/MobileTopBarVideoFilters.svelte";
  import WorkspaceContentPanel from "$lib/components/workspace/WorkspaceContentPanel.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import MobileHomeBrowseOverlay from "$lib/components/mobile/MobileHomeBrowseOverlay.svelte";
  import WorkspaceDesktopTabNav from "$lib/components/workspace/WorkspaceDesktopTabNav.svelte";
  import WorkspaceDesktopTopBar from "$lib/components/workspace/WorkspaceDesktopTopBar.svelte";
  import WorkspaceShell from "$lib/components/workspace/WorkspaceShell.svelte";
  import WorkspaceSidebar from "$lib/components/workspace/WorkspaceSidebar.svelte";
  import FeatureGuide from "$lib/components/FeatureGuide.svelte";
  import { setFeatureGuideSuppressesAuthRequiredNotice } from "$lib/auth-required-notice";
  import { createHomeWorkspacePage } from "$lib/workspace/home-workspace.svelte";

  const hw = createHomeWorkspacePage();

  $effect(() => {
    setFeatureGuideSuppressesAuthRequiredNotice(hw.guideOpen);
    return () => {
      setFeatureGuideSuppressesAuthRequiredNotice(false);
    };
  });

  // ---------------------------------------------------------------------------
  // Mobile back-swipe guard
  //
  // On mobile (web + Tauri Android), the OS back gesture fires history.back(),
  // which would navigate away from the app (to the login page or close it).
  // We prevent that by maintaining a synthetic history entry while on this page.
  //
  // Strategy:
  //   1. On mount push one guard entry so there is always an entry to pop back
  //      to before the browser would try to leave the page.
  //   2. When the guard entry is popped (popstate) and we are in video view,
  //      open the browse overlay. If already in browse view, the guard just
  //      re-pushes itself so the next back also stays on this page.
  // ---------------------------------------------------------------------------
  const BACK_GUARD_STATE = { dastill_back_guard: true };

  onMount(() => {
    // Push the initial guard entry on top of the current history entry.
    history.pushState(BACK_GUARD_STATE, "");

    function handlePopState() {
      if (!hw.mobileBrowseOpen) {
        // Video view → back means "show channel list".
        hw.openMobileBrowse();
      }
      // Always re-push the guard so subsequent back presses are also caught.
      history.pushState(BACK_GUARD_STATE, "");
    }

    window.addEventListener("popstate", handlePopState);
    return () => {
      window.removeEventListener("popstate", handlePopState);
    };
  });
</script>

<WorkspaceShell
  currentSection="workspace"
  aiIndicator={hw.aiIndicator}
  onOpenGuide={hw.openGuide}
>
  {#snippet sidebar(shell)}
    <WorkspaceSidebar
      videoListMode="per_channel_preview"
      previewSessionKey="workspace-sidebar-navigation"
      addSourceErrorMessage={hw.errorMessage}
      initialChannelPreviews={hw.page.data.channelPreviews ?? {}}
      initialChannelPreviewsFilterKey={hw.page.data.channelPreviewsFilterKey ??
        "all:all:default"}
      previewScope={{ kind: "default" }}
      shell={{
        collapsed: shell.collapsed,
        width: shell.width,
        mobileVisible: shell.mobileVisible ?? false,
        onToggleCollapse: shell.toggle,
      }}
      channelState={{
        ...hw.sidebarState.channelState,
        canDeleteChannels: hw.canManageLibrary,
      }}
      channelActions={{
        ...hw.sidebarState.channelActions,
        onDeleteChannel: hw.handleDeleteChannel,
        onDeleteAccessRequired: hw.openDeleteAccessPrompt,
      }}
      videoState={hw.sidebarState.videoState}
      videoActions={hw.sidebarState.videoActions}
      videoAcknowledgeSync={hw.videoAcknowledgeSync}
      onChannelSyncDateSaved={hw.handleChannelSyncDateSaved}
      onChannelPreviewSnapshotLoaded={hw.cacheChannelPreviewSnapshot}
    />
  {/snippet}
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav
      showBackInsteadOfMenu={!hw.mobileBrowseOpen &&
        Boolean(hw.selectedVideoId)}
      onBack={hw.openMobileBrowse}
    >
      {#snippet trailing()}
        <MobileTopBarVideoFilters
          visible={hw.mobileBrowseOpen}
          videoTypeFilter={hw.sidebarState.videoState.videoTypeFilter}
          acknowledgedFilter={hw.sidebarState.videoState.acknowledgedFilter}
          disabled={hw.browseFilterDisabled}
          onSelectVideoType={hw.onBrowseVideoTypeFilterChange}
          onSelectAcknowledged={hw.onBrowseAcknowledgedFilterChange}
          onClearAllFilters={hw.clearBrowseVideoFilters}
        />
      {/snippet}
    </MobileYouTubeTopNav>
  {/snippet}
  {#snippet tabNav()}
    <WorkspaceDesktopTabNav
      contentMode={hw.contentMode}
      onSetMode={hw.setMode}
    />
  {/snippet}
  {#snippet topBar()}
    <WorkspaceDesktopTopBar
      contentMode={hw.contentMode}
      selectedVideoId={hw.selectedVideoId}
      selectedVideo={hw.selectedVideo}
      contentText={hw.workspaceContentState.contentText}
      loadingContent={hw.loadingContent}
      editing={hw.editing}
      hasUpdatedTranscript={hw.hasUpdatedTranscript}
      formattingContent={hw.formattingContent}
      formattingVideoId={hw.formattingVideoId}
      regeneratingSummaryVideoIds={hw.regeneratingSummaryVideoIds}
      revertingContent={hw.revertingContent}
      revertingVideoId={hw.revertingVideoId}
      resettingVideo={hw.resettingVideo}
      resettingVideoId={hw.resettingVideoId}
      aiAvailable={hw.aiAvailable ?? false}
      canRevertTranscript={hw.canRevertTranscript}
      selectedVideoYoutubeUrl={hw.selectedVideoYoutubeUrl}
      draft={hw.draft}
      selectedVideoAcknowledged={hw.selectedVideo?.acknowledged ?? false}
      onEdit={hw.startEdit}
      onCancel={hw.cancelEdit}
      onSave={hw.saveEdit}
      onFormat={hw.cleanFormatting}
      onRegenerate={hw.regenerateSummaryContent}
      onRevert={hw.revertToOriginalTranscript}
      onRequestResetVideo={hw.openResetVideoConfirmation}
      onDraftChange={hw.setDraft}
      onAcknowledgeToggle={hw.toggleAcknowledge}
    >
      {#snippet searchBar()}
        {#if hw.WorkspaceSearchBarComponent}
          <hw.WorkspaceSearchBarComponent
            initialSearchStatus={hw.searchStatus}
            onSearchResultSelect={hw.handleSearchResultSelection}
          />
        {/if}
      {/snippet}
    </WorkspaceDesktopTopBar>
  {/snippet}

  <MobileHomeBrowseOverlay
    open={hw.mobileBrowseOpen}
    channels={hw.sidebarState.channels}
    selectedChannelId={hw.sidebarState.selectedChannelId}
    onSelectChannel={(channelId) => {
      void hw.sidebarState.selectChannel(channelId);
    }}
    onClose={hw.closeMobileBrowse}
    channelState={{
      ...hw.sidebarState.channelState,
      canDeleteChannels: hw.canManageLibrary,
    }}
    channelActions={{
      ...hw.sidebarState.channelActions,
      onDeleteChannel: hw.handleDeleteChannel,
      onDeleteAccessRequired: hw.openDeleteAccessPrompt,
    }}
    videoState={{
      ...hw.sidebarState.videoState,
      historyExhausted: hw.sidebarState.historyExhausted,
      backfillingHistory: hw.sidebarState.backfillingHistory,
    }}
    videoActions={{
      ...hw.sidebarState.videoActions,
      onLoadMoreVideos: hw.loadMoreVideos,
    }}
    canDeleteChannels={hw.canManageLibrary}
    addSourceErrorMessage={hw.errorMessage}
    onChannelSyncDateSaved={hw.handleChannelSyncDateSaved}
  />

  <WorkspaceContentPanel
    selection={hw.workspaceContentSelection}
    content={hw.workspaceContentState}
    actions={hw.workspaceContentActions}
    overlays={hw.workspaceOverlaysState}
    overlayActions={hw.workspaceOverlaysActions}
  />

  {#if hw.addSourceFeedback && !hw.addSourceFeedbackDismissed}
    <AddSourceFeedbackToast
      feedback={hw.addSourceFeedback}
      onDismiss={hw.dismissAddSourceFeedback}
      onAction={hw.openAddSourceFeedbackTarget}
    />
  {/if}

  {#if hw.guideOpen}
    <FeatureGuide
      open={hw.guideOpen}
      step={hw.guideStep}
      steps={hw.tourSteps}
      docsUrl={hw.DOCS_URL}
      onClose={hw.closeGuide}
      onStep={hw.setGuideStep}
    />
  {/if}

  <VocabularyReplacementModal
    show={Boolean(hw.vocabularyModalSource)}
    source={hw.vocabularyModalSource ?? ""}
    value={hw.vocabularyModalValue}
    busy={hw.creatingVocabularyReplacement}
    onValueChange={hw.setVocabularyModalValue}
    onConfirm={() => void hw.confirmVocabularyReplacement()}
    onCancel={hw.closeVocabularyModal}
  />
</WorkspaceShell>
