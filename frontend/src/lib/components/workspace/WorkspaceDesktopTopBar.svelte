<script lang="ts">
  import type { Snippet } from "svelte";
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  let {
    selectedVideoId,
    loadingContent,
    editing,
    hasUpdatedTranscript,
    formattingContent,
    formattingVideoId,
    regeneratingSummaryVideoIds,
    revertingContent,
    revertingVideoId,
    resettingVideo,
    resettingVideoId,
    aiAvailable,
    canRevertTranscript,
    contentMode,
    selectedVideoYoutubeUrl,
    draft,
    selectedVideoAcknowledged,
    onEdit,
    onCancel,
    onSave,
    onFormat,
    onRegenerate,
    onRevert,
    onRequestResetVideo,
    onDraftChange,
    onAcknowledgeToggle,
    searchBar,
  }: {
    selectedVideoId: string | null;
    loadingContent: boolean;
    editing: boolean;
    hasUpdatedTranscript: boolean;
    formattingContent: boolean;
    formattingVideoId: string | null;
    regeneratingSummaryVideoIds: string[];
    revertingContent: boolean;
    revertingVideoId: string | null;
    resettingVideo: boolean;
    resettingVideoId: string | null;
    aiAvailable: boolean;
    canRevertTranscript: boolean;
    contentMode: WorkspaceContentMode;
    selectedVideoYoutubeUrl: string | null;
    draft: string;
    selectedVideoAcknowledged: boolean;
    onEdit: () => void;
    onCancel: () => void;
    onSave: () => void | Promise<void>;
    onFormat: () => void | Promise<void>;
    onRegenerate: () => void | Promise<void>;
    onRevert: () => void | Promise<void>;
    onRequestResetVideo: () => void;
    onDraftChange: (value: string) => void;
    onAcknowledgeToggle: () => void | Promise<void>;
    searchBar?: Snippet;
  } = $props();
</script>

<div class="flex min-w-0 flex-1 items-center justify-end gap-3">
  {#if searchBar}
    {@render searchBar()}
  {/if}
  {#if selectedVideoId && !loadingContent && !editing}
    <div
      class="flex items-center border-l border-[var(--border-soft)] pl-3"
      id="content-actions"
    >
      <ContentEditor
        editing={false}
        busy={loadingContent}
        {aiAvailable}
        formatting={formattingContent && formattingVideoId === selectedVideoId}
        regenerating={Boolean(
          selectedVideoId &&
          regeneratingSummaryVideoIds.includes(selectedVideoId),
        )}
        reverting={revertingContent && revertingVideoId === selectedVideoId}
        resetting={resettingVideo && resettingVideoId === selectedVideoId}
        showFormatAction={contentMode === "transcript"}
        showRegenerateAction={contentMode === "summary"}
        showRevertAction={hasUpdatedTranscript}
        showEditAction={contentMode === "transcript" ||
          contentMode === "summary"}
        canRevert={canRevertTranscript}
        youtubeUrl={selectedVideoYoutubeUrl}
        value={draft}
        acknowledged={selectedVideoAcknowledged}
        acknowledgeToggleId="mark-read-toggle"
        minimalActionChrome
        {onEdit}
        {onCancel}
        {onSave}
        {onFormat}
        {onRegenerate}
        {onRevert}
        onReset={onRequestResetVideo}
        onChange={onDraftChange}
        {onAcknowledgeToggle}
      />
    </div>
  {/if}
</div>
