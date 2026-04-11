<script lang="ts">
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import type { Video } from "$lib/types";
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  let {
    mobileBackInTopBar = false,
    contentMode,
    selectedVideoId,
    selectedVideo = null as Video | null,
    loadingContent,
    editing,
    aiAvailable,
    formattingContent,
    formattingVideoId,
    summaryRegeneratingForSelection,
    revertingContent,
    revertingVideoId,
    resettingVideo,
    resettingVideoId,
    showRevertTranscriptAction,
    canRevertTranscript,
    selectedVideoYoutubeUrl,
    draft,
    onBack,
    onStartEdit,
    onCancelEdit,
    onSaveEdit,
    onCleanFormatting,
    onRegenerateSummary,
    onRevertTranscript,
    onRequestResetVideo,
    onDraftChange,
    onToggleAcknowledge,
  }: {
    mobileBackInTopBar?: boolean;
    contentMode: WorkspaceContentMode;
    selectedVideoId: string | null;
    selectedVideo?: Video | null;
    loadingContent: boolean;
    editing: boolean;
    aiAvailable: boolean;
    formattingContent: boolean;
    formattingVideoId: string | null;
    summaryRegeneratingForSelection: boolean;
    revertingContent: boolean;
    revertingVideoId: string | null;
    resettingVideo: boolean;
    resettingVideoId: string | null;
    showRevertTranscriptAction: boolean;
    canRevertTranscript: boolean;
    selectedVideoYoutubeUrl: string | null;
    draft: string;
    onBack: () => void;
    onStartEdit: () => void;
    onCancelEdit: () => void;
    onSaveEdit: () => void | Promise<void>;
    onCleanFormatting: () => void | Promise<void>;
    onRegenerateSummary: () => void | Promise<void>;
    onRevertTranscript: () => void | Promise<void>;
    onRequestResetVideo: () => void;
    onDraftChange: (value: string) => void;
    onToggleAcknowledge: () => void | Promise<void>;
  } = $props();
</script>

<div
  class="flex items-center justify-between gap-3 px-4 pb-1 pt-2 sm:px-6 lg:hidden"
>
  <h2 class="sr-only">Content actions</h2>
  {#if !mobileBackInTopBar}
    <button
      type="button"
      class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
      onclick={onBack}
      aria-label="Back"
    >
      <ChevronIcon direction="left" size={18} strokeWidth={2.2} />
    </button>
  {/if}

  {#if selectedVideoId && !loadingContent && !editing}
    <div
      id="content-actions"
      class="relative z-20 ml-auto flex h-10 items-center justify-end"
    >
      <ContentEditor
        editing={false}
        busy={loadingContent}
        {aiAvailable}
        formatting={formattingContent && formattingVideoId === selectedVideoId}
        regenerating={summaryRegeneratingForSelection}
        reverting={revertingContent && revertingVideoId === selectedVideoId}
        resetting={resettingVideo && resettingVideoId === selectedVideoId}
        showFormatAction={contentMode === "transcript"}
        showRegenerateAction={contentMode === "summary"}
        showRevertAction={showRevertTranscriptAction}
        showEditAction={contentMode === "transcript" ||
          contentMode === "summary"}
        canRevert={canRevertTranscript}
        youtubeUrl={selectedVideoYoutubeUrl}
        value={draft}
        acknowledged={selectedVideo?.acknowledged ?? false}
        onEdit={onStartEdit}
        onCancel={onCancelEdit}
        onSave={onSaveEdit}
        onFormat={onCleanFormatting}
        onRegenerate={onRegenerateSummary}
        onRevert={onRevertTranscript}
        onReset={onRequestResetVideo}
        onChange={(value) => onDraftChange(value)}
        onAcknowledgeToggle={onToggleAcknowledge}
      />
    </div>
  {/if}
</div>
