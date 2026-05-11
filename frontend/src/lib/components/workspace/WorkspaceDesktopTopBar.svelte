<script lang="ts">
  import type { Snippet } from "svelte";
  import ContentEditor from "$lib/components/ContentEditor.svelte";
  import CopyIcon from "$lib/components/icons/CopyIcon.svelte";
  import CheckIcon from "$lib/components/icons/CheckIcon.svelte";
  import QueueStatusPopover from "$lib/components/workspace/QueueStatusPopover.svelte";
  import type { Video } from "$lib/types";
  import type { WorkspaceContentMode } from "$lib/workspace/types";
  import {
    generateSummaryAudio,
    readSummaryAudioSession,
    subscribeToSummaryAudioSession,
    type SummaryAudioStatus,
  } from "$lib/workspace/summary-audio-session";
  import { createApiRequestInit, resolveApiUrl } from "$lib/api/client";

  let {
    selectedVideoId,
    selectedVideo = null as Video | null,
    contentText = "",
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
    selectedVideo?: Video | null;
    contentText?: string;
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
  let copyActionLabel = $derived(
    contentMode === "transcript" ? "Copy transcript text" : "Copy summary text",
  );
  let copiedLabel = $derived(
    contentMode === "transcript" ? "Copied transcript" : "Copied summary",
  );

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

  const minimalBtn =
    "inline-flex h-8 w-8 items-center justify-center rounded-md text-[var(--soft-foreground)] opacity-70 transition-colors hover:text-[var(--foreground)] hover:opacity-100 disabled:cursor-not-allowed disabled:opacity-20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40";
</script>

<div class="flex min-w-0 flex-1 items-center justify-end gap-3">
  {#if searchBar}
    {@render searchBar()}
  {/if}
  <QueueStatusPopover />
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
      >
        {#snippet extraViewActions()}
          {#if contentMode === "summary" && audioStatus === "missing"}
            <button
              type="button"
              class={minimalBtn}
              aria-label="Generate audio"
              onclick={generateAudio}
              disabled={selectedVideo?.summary_status !== "ready"}
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
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
              class={minimalBtn}
              aria-label={copyState === "copied"
                ? copiedLabel
                : copyState === "error"
                  ? "Copy failed"
                  : copyActionLabel}
              onclick={copyContent}
            >
              {#if copyState === "copied"}
                <CheckIcon
                  size={16}
                  strokeWidth={1.8}
                  className="text-[var(--accent)]"
                />
              {:else}
                <CopyIcon size={15} strokeWidth={1.8} />
              {/if}
            </button>
          {/if}
        {/snippet}
      </ContentEditor>
    </div>
  {/if}
</div>
