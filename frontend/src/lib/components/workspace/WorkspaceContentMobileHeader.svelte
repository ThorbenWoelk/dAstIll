<script lang="ts">
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import type { Video } from "$lib/types";
  import { WORKSPACE_CONTENT_MODE_ORDER } from "$lib/workspace/navigation";
  import type { WorkspaceContentMode } from "$lib/workspace/types";

  const CONTENT_MODE_LABELS: Record<WorkspaceContentMode, string> = {
    transcript: "Transcript",
    summary: "Summary",
    highlights: "Highlights",
    info: "Info",
  };

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
    onSetMode,
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
    onSetMode: (mode: WorkspaceContentMode) => void | Promise<void>;
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
  class="flex flex-col gap-3 px-4 pb-1 pt-3 max-lg:pb-1 max-lg:pt-3 sm:px-6 lg:hidden"
>
  <h2 class="sr-only">Display Content</h2>
  {#if !mobileBackInTopBar}
    <div class="flex items-center gap-3">
      <button
        type="button"
        class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-[var(--soft-foreground)] transition-colors hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)]"
        onclick={onBack}
        aria-label="Back"
      >
        <ChevronIcon direction="left" size={18} strokeWidth={2.2} />
      </button>
    </div>
  {/if}
  <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
    <div class="min-w-0 flex-1" id="workspace-tabs-mobile">
      <div
        data-mobile-content-tabs
        class="custom-scrollbar flex min-w-0 items-center gap-1 overflow-x-auto [-ms-overflow-style:none] [scroll-padding-inline:0.5rem] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      >
        {#each WORKSPACE_CONTENT_MODE_ORDER as mode}
          <button
            type="button"
            data-workspace-content-tab={mode}
            class={`shrink-0 rounded-full px-3 py-1.5 text-[11px] font-bold uppercase tracking-[0.1em] transition-colors ${
              contentMode === mode
                ? "bg-[var(--accent-wash-strong)] text-[var(--accent-strong)] shadow-sm"
                : "text-[var(--soft-foreground)] opacity-65 hover:bg-[var(--accent-wash)] hover:text-[var(--foreground)] hover:opacity-100"
            }`}
            aria-pressed={contentMode === mode}
            aria-label={`${CONTENT_MODE_LABELS[mode]} tab`}
            onclick={() => void onSetMode(mode)}
          >
            {CONTENT_MODE_LABELS[mode]}
          </button>
        {/each}
      </div>
    </div>

    {#if selectedVideoId && !loadingContent && !editing}
      <div
        id="content-actions"
        class="relative z-20 flex h-10 items-center justify-end self-end lg:shrink-0 lg:self-auto"
      >
        <ContentEditor
          editing={false}
          busy={loadingContent}
          {aiAvailable}
          formatting={formattingContent &&
            formattingVideoId === selectedVideoId}
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
</div>
