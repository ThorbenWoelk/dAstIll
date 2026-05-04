<script lang="ts">
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import CopyIcon from "$lib/components/icons/CopyIcon.svelte";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import type { Video } from "$lib/types";
  import type { WorkspaceContentMode } from "$lib/workspace/types";
  import {
    generateSummaryAudio,
    readSummaryAudioSession,
    subscribeToSummaryAudioSession,
    type SummaryAudioStatus,
  } from "$lib/workspace/summary-audio-session";
  import { createApiRequestInit, resolveApiUrl } from "$lib/api-client";

  let {
    mobileBackInTopBar = false,
    contentMode,
    selectedVideoId,
    selectedVideo = null as Video | null,
    contentText = "",
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
    contentText?: string;
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

  // Copy state
  let copyState = $state<"idle" | "copied" | "error">("idle");
  let copyResetTimer: ReturnType<typeof setTimeout> | null = null;

  let canCopy = $derived(
    (contentMode === "summary" || contentMode === "transcript") &&
      contentText.trim().length > 0,
  );
  let copyText = $derived.by(() => {
    const title = selectedVideo?.title.trim() ?? "";
    const body = contentText.trim();
    return title ? `${title}\n\n${body}` : body;
  });

  $effect(() => {
    contentMode;
    copyText;
    if (copyResetTimer) {
      clearTimeout(copyResetTimer);
      copyResetTimer = null;
    }
    copyState = "idle";
  });

  $effect(() => {
    return () => {
      if (copyResetTimer) {
        clearTimeout(copyResetTimer);
        copyResetTimer = null;
      }
    };
  });

  async function copyContent() {
    if (!canCopy) return;
    try {
      await navigator.clipboard.writeText(copyText);
      copyState = "copied";
    } catch {
      copyState = "error";
    }
    if (copyResetTimer) clearTimeout(copyResetTimer);
    copyResetTimer = setTimeout(() => {
      copyState = "idle";
      copyResetTimer = null;
    }, 2000);
  }

  // Audio state
  let audioStatus = $state<SummaryAudioStatus>("missing");
  let unsubAudio: (() => void) | null = null;

  $effect(() => {
    unsubAudio?.();
    unsubAudio = null;
    if (!selectedVideoId || contentMode !== "summary") {
      audioStatus = "missing";
      return;
    }
    audioStatus = readSummaryAudioSession(selectedVideoId).status;
    unsubAudio = subscribeToSummaryAudioSession(selectedVideoId, () => {
      if (selectedVideoId) {
        audioStatus = readSummaryAudioSession(selectedVideoId).status;
      }
    });
    return () => {
      unsubAudio?.();
      unsubAudio = null;
    };
  });

  async function generateAudio() {
    if (!selectedVideoId) return;
    const videoId = selectedVideoId;
    await generateSummaryAudio(videoId, async () =>
      fetch(
        resolveApiUrl(`/api/videos/${videoId}/summary/audio`),
        await createApiRequestInit(
          { method: "POST" },
          { includeJsonContentType: false },
        ),
      ),
    );
  }

  const ghostBtn =
    "inline-flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] border border-transparent text-[var(--soft-foreground)] transition-all hover:border-[var(--border-soft)] hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] disabled:cursor-not-allowed disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40";
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
      class={`relative z-20 flex h-10 items-center ${
        mobileBackInTopBar
          ? "w-full justify-stretch"
          : "ml-auto flex-1 justify-end"
      }`}
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
      >
        {#snippet extraViewActions()}
          {#if contentMode === "summary" && audioStatus === "missing"}
            <button
              type="button"
              class={ghostBtn}
              aria-label="Generate audio"
              onclick={generateAudio}
              disabled={selectedVideo?.summary_status !== "ready"}
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path d="M3 18v-6a9 9 0 0 1 18 0v6" />
                <path
                  d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"
                />
              </svg>
            </button>
          {/if}
          {#if canCopy}
            <button
              type="button"
              class={ghostBtn}
              aria-label={copyState === "copied"
                ? "Copied"
                : copyState === "error"
                  ? "Copy failed"
                  : "Copy text"}
              onclick={copyContent}
            >
              {#if copyState === "copied"}
                <CheckIcon
                  size={15}
                  strokeWidth={2}
                  className="text-[var(--accent)]"
                />
              {:else}
                <CopyIcon size={14} strokeWidth={2} />
              {/if}
            </button>
          {/if}
        {/snippet}
      </ContentEditor>
    </div>
  {/if}
</div>
