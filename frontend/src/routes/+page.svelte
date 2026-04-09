<script lang="ts">
  import AddSourceFeedbackToast from "$lib/components/AddSourceFeedbackToast.svelte";
  import VocabularyReplacementModal from "$lib/components/VocabularyReplacementModal.svelte";
  import WorkspaceContentPanel from "$lib/components/workspace/WorkspaceContentPanel.svelte";
  import MobileYouTubeTopNav from "$lib/components/mobile/MobileYouTubeTopNav.svelte";
  import MobileHomeBrowseOverlay from "$lib/components/mobile/MobileHomeBrowseOverlay.svelte";
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
    />
  {/snippet}
  {#snippet mobileTopBar()}
    <MobileYouTubeTopNav
      showBackInsteadOfMenu={!hw.mobileBrowseOpen &&
        Boolean(hw.selectedVideoId)}
      onBack={hw.openMobileBrowse}
    >
      {#snippet trailing()}
        <div class="w-10 shrink-0" aria-hidden="true"></div>
      {/snippet}
    </MobileYouTubeTopNav>
  {/snippet}
  {#snippet topBar()}
    <WorkspaceDesktopTopBar
      contentMode={hw.contentMode}
      onSetMode={hw.setMode}
      selectedVideoId={hw.selectedVideoId}
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
    previewSessionKey="workspace-sidebar-navigation"
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
