<script lang="ts">
  import type { Snippet } from "svelte";
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import {
    goHintKeyForWorkspaceContentMode,
    WORKSPACE_CONTENT_MODE_ORDER,
  } from "$lib/workspace/navigation";
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  let {
    contentMode,
    onSetMode,
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
    contentMode: WorkspaceContentMode;
    onSetMode: (mode: WorkspaceContentMode) => void | Promise<void>;
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

<div class="flex items-center gap-6" id="workspace-tabs-desktop">
  {#each WORKSPACE_CONTENT_MODE_ORDER as mode}
    <button
      type="button"
      data-workspace-content-tab={mode}
      data-go-hint-key={goHintKeyForWorkspaceContentMode(mode)}
      class={`-mb-px inline-flex h-6 items-center border-b-2 text-[11px] font-bold uppercase tracking-[0.12em] transition-colors ${
        contentMode === mode
          ? "border-[var(--accent)] text-[var(--accent-strong)]"
          : "border-transparent text-[var(--soft-foreground)] opacity-75 hover:text-[var(--foreground)] hover:opacity-100"
      }`}
      aria-pressed={contentMode === mode}
      onclick={() => void onSetMode(mode)}
    >
      {mode === "transcript"
        ? "Transcript"
        : mode === "summary"
          ? "Summary"
          : mode === "highlights"
            ? "Highlights"
            : "Info"}
    </button>
  {/each}

  {#if selectedVideoId && !loadingContent && !editing}
    <div
      class="ml-2 border-l border-[var(--border-soft)] pl-4"
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
<div class="flex min-w-0 flex-1 items-center justify-end gap-3">
  {#if searchBar}
    {@render searchBar()}
  {/if}
</div>
