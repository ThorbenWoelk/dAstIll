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

<div class="flex min-w-0 items-center gap-2" id="workspace-tabs-desktop">
  {#each WORKSPACE_CONTENT_MODE_ORDER as mode}
    <button
      type="button"
      data-workspace-content-tab={mode}
      data-go-hint-key={goHintKeyForWorkspaceContentMode(mode)}
      class={`inline-flex h-8 items-center rounded-full px-3.5 text-[10px] font-bold uppercase tracking-[0.12em] transition-all ${
        contentMode === mode
          ? "bg-[var(--foreground)] text-[var(--background)] shadow-sm"
          : "text-[var(--soft-foreground)] opacity-80 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100"
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
      class="ml-2 flex items-center border-l border-[var(--border-soft)] pl-4"
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
  <a
    href="/mini"
    class="inline-flex h-8 items-center rounded-full border border-[var(--border-soft)] px-3 text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--soft-foreground)] transition-all hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
    data-sveltekit-preload-data="tap"
    data-sveltekit-preload-code="viewport"
  >
    Mini
  </a>
  {#if searchBar}
    {@render searchBar()}
  {/if}
</div>
