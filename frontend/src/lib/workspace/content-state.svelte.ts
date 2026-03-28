import { SvelteMap } from "svelte/reactivity";
import {
  ensureTranscript,
  getSummary,
  ensureVideoInfo,
  updateTranscript,
  updateSummary,
  regenerateSummary as apiRegenerateSummary,
  cleanTranscriptFormatting,
} from "$lib/api";
import {
  resolveTranscriptPresentation,
  resolveSummaryQualityPresentation,
  stripContentPrefix,
} from "$lib/workspace/content";
import { deriveSummaryTrackingId } from "$lib/workspace/summary-tracking-id";
import {
  openSummarySession,
  closeSummarySession,
  isSummarySessionOpen,
} from "$lib/analytics/read-time";
import {
  buildFormattingAttemptSummary,
  clearFormattingFeedbackState,
  resetSummaryQualityState,
} from "$lib/workspace/formatting";
import type {
  Transcript,
  TranscriptRenderMode,
  VideoInfo as VideoInfoPayload,
  Summary as SummaryPayload,
  ContentStatus,
} from "$lib/types";
import type { WorkspaceContentMode } from "$lib/workspace/types";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { track } from "$lib/analytics/tracker";

export type ContentCacheEntry = {
  transcript?: {
    text: string;
    renderMode: TranscriptRenderMode;
  };
  summary?: {
    text: string;
    quality: SummaryPayload;
    trackingId: string;
  };
  info?: VideoInfoPayload;
};

export function createContentState(options: {
  getSelectedVideoId: () => string | null;
  getSelectedChannelId: () => string | null;
  setVideoStatus: (
    videoId: string,
    transcriptStatus: ContentStatus | undefined,
    summaryStatus: ContentStatus | undefined,
  ) => void;
  initialContentMode?: WorkspaceContentMode;
}) {
  let loadingContent = $state(false);
  let contentMode = $state<WorkspaceContentMode>(
    options.initialContentMode ?? "info",
  );
  let contentText = $state("");
  let transcriptRenderMode = $state<TranscriptRenderMode>("plain_text");
  let draftTranscriptRenderMode = $state<TranscriptRenderMode>("plain_text");
  let editing = $state(false);
  let draft = $state("");
  let videoInfo = $state<VideoInfoPayload | null>(null);

  let summaryQualityScore = $state<number | null>(null);
  let summaryQualityNote = $state<string | null>(null);
  let summaryModelUsed = $state<string | null>(null);
  let summaryQualityModelUsed = $state<string | null>(null);

  let formattingContent = $state(false);
  let formattingVideoId = $state<string | null>(null);
  let regeneratingSummaryVideoIds = $state<string[]>([]);
  let revertingContent = $state(false);
  let revertingVideoId = $state<string | null>(null);
  let resettingVideo = $state(false);
  let resettingVideoId = $state<string | null>(null);

  let formattingNotice = $state<string | null>(null);
  let formattingNoticeVideoId = $state<string | null>(null);
  let formattingNoticeTone = $state<"info" | "success" | "warning">("info");
  let formattingAttemptsUsed = $state<number | null>(null);
  let formattingAttemptsMax = $state<number | null>(null);
  let formattingAttemptsVideoId = $state<string | null>(null);

  const originalTranscriptByVideoId = $state<Record<string, string>>({});
  const contentCache = new SvelteMap<string, ContentCacheEntry>();

  let contentRequestSeq = 0;
  let activeContentRequestId = 0;
  let formattingRequestSeq = 0;
  let activeFormattingRequest = 0;

  function resetSummaryQuality() {
    const nextState = resetSummaryQualityState();
    summaryQualityScore = nextState.score;
    summaryQualityNote = nextState.note;
    summaryModelUsed = nextState.modelUsed;
    summaryQualityModelUsed = nextState.qualityModelUsed;
  }

  function applySummaryQuality(summary: SummaryPayload) {
    const presentation = resolveSummaryQualityPresentation(summary);
    summaryQualityScore = presentation.score;
    summaryQualityNote = presentation.note;
    summaryModelUsed = presentation.modelUsed;
    summaryQualityModelUsed = presentation.qualityModelUsed;
  }

  function syncSummaryTrackingSession(
    summary: SummaryPayload,
    videoId: string,
    channelId: string,
  ) {
    const trackingId = deriveSummaryTrackingId(summary);
    if (isSummarySessionOpen(videoId, trackingId)) {
      return trackingId;
    }

    closeSummarySession();
    openSummarySession({
      video_id: videoId,
      channel_id: channelId,
      summary_id: trackingId,
    });
    return trackingId;
  }

  function cacheLoadedSummary(summary: SummaryPayload, videoId: string) {
    const summaryText = stripContentPrefix(
      summary.content || "Summary unavailable.",
    );
    const channelId = options.getSelectedChannelId();
    const trackingId = channelId
      ? syncSummaryTrackingSession(summary, videoId, channelId)
      : deriveSummaryTrackingId(summary);
    const prev = contentCache.get(videoId);
    contentCache.set(videoId, {
      ...prev,
      summary: {
        text: summaryText,
        quality: summary,
        trackingId,
      },
    });
    return summaryText;
  }

  function invalidateContentCache(
    videoId: string,
    mode?: "transcript" | "summary" | "info",
  ) {
    if (!mode) {
      contentCache.delete(videoId);
      return;
    }
    const entry = contentCache.get(videoId);
    if (entry) {
      delete entry[mode];
    }
  }

  function isCurrentContentRequest(
    requestId: number,
    targetVideoId: string,
    targetMode: WorkspaceContentMode,
  ) {
    return (
      activeContentRequestId === requestId &&
      options.getSelectedVideoId() === targetVideoId &&
      contentMode === targetMode
    );
  }

  async function loadContent() {
    const targetVideoId = options.getSelectedVideoId();
    if (!targetVideoId) return;
    const targetMode = contentMode;
    const requestId = ++contentRequestSeq;
    activeContentRequestId = requestId;

    // Check cache first
    const cached = contentCache.get(targetVideoId);
    if (cached) {
      if (targetMode === "transcript" && cached.transcript !== undefined) {
        contentText = cached.transcript.text;
        transcriptRenderMode = cached.transcript.renderMode;
        draft = contentText;
        draftTranscriptRenderMode = transcriptRenderMode;
        resetSummaryQuality();
        videoInfo = null;
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
      if (targetMode === "summary" && cached.summary) {
        contentText = cached.summary.text;
        applySummaryQuality(cached.summary.quality);
        const channelId = options.getSelectedChannelId();
        if (channelId) {
          syncSummaryTrackingSession(
            cached.summary.quality,
            targetVideoId,
            channelId,
          );
        }
        videoInfo = null;
        draft = contentText;
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
      if (targetMode === "info" && cached.info) {
        videoInfo = cached.info;
        contentText = "";
        resetSummaryQuality();
        draft = contentText;
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
      if (targetMode === "highlights") {
        // Highlights are handled by parent component for now as they need highlight state
        contentText = "";
        resetSummaryQuality();
        videoInfo = null;
        draft = "";
        loadingContent = false;
        activeContentRequestId = 0;
        return;
      }
    }

    loadingContent = true;

    try {
      if (targetMode === "transcript") {
        const channelId = options.getSelectedChannelId();
        if (channelId) {
          track({
            event: "transcript_ensure_requested",
            video_id: targetVideoId,
            channel_id: channelId,
          });
        }
        let transcriptSuccess = false;
        let transcript: Transcript | undefined;
        try {
          transcript = await ensureTranscript(targetVideoId);
          transcriptSuccess = true;
        } catch (error) {
          if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
            return;
          options.setVideoStatus(targetVideoId, "failed", undefined);
          throw error;
        } finally {
          if (channelId) {
            track({
              event: "transcript_ensure_completed",
              video_id: targetVideoId,
              channel_id: channelId,
              success: transcriptSuccess,
            });
          }
        }
        if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
          return;
        const presentation = resolveTranscriptPresentation(transcript!);
        const originalTranscript = presentation.originalText;
        contentText = presentation.content;
        transcriptRenderMode = presentation.renderMode;
        draftTranscriptRenderMode = presentation.renderMode;
        if (!(targetVideoId in originalTranscriptByVideoId)) {
          originalTranscriptByVideoId[targetVideoId] = originalTranscript;
        }
        // Cache the transcript
        const entry = contentCache.get(targetVideoId) ?? {};
        entry.transcript = {
          text: presentation.content,
          renderMode: presentation.renderMode,
        };
        contentCache.set(targetVideoId, entry);
        resetSummaryQuality();
        videoInfo = null;
        options.setVideoStatus(targetVideoId, "ready", undefined);
      } else if (targetMode === "summary") {
        try {
          const summary = await getSummary(targetVideoId);
          if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
            return;
          contentText = cacheLoadedSummary(summary, targetVideoId);
          applySummaryQuality(summary);
          videoInfo = null;
        } catch (error) {
          if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
            return;
          const message = (error as Error).message || "";
          if (message.includes("Summary not found")) {
            contentText = "";
            resetSummaryQuality();
            videoInfo = null;
          } else if (presentAuthRequiredNoticeIfNeeded(error)) {
            return;
          } else {
            throw error;
          }
        }
      } else if (targetMode === "info") {
        const info = await ensureVideoInfo(targetVideoId);
        if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
          return;
        videoInfo = info;
        contentText = "";
        const entry = contentCache.get(targetVideoId) ?? {};
        entry.info = info;
        contentCache.set(targetVideoId, entry);
        resetSummaryQuality();
      }

      if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
        return;
      draft = contentText;
      if (targetMode === "transcript") {
        draftTranscriptRenderMode = transcriptRenderMode;
      }
    } finally {
      if (activeContentRequestId === requestId) {
        loadingContent = false;
        activeContentRequestId = 0;
      }
    }
  }

  function clearFormattingFeedback() {
    const nextState = clearFormattingFeedbackState();
    formattingNotice = nextState.formattingNotice;
    formattingNoticeVideoId = nextState.formattingNoticeVideoId;
    formattingAttemptsUsed = nextState.formattingAttemptsUsed;
    formattingAttemptsMax = nextState.formattingAttemptsMax;
    formattingAttemptsVideoId = nextState.formattingAttemptsVideoId;
  }

  return {
    get loadingContent() {
      return loadingContent;
    },
    get contentMode() {
      return contentMode;
    },
    set contentMode(v) {
      contentMode = v;
    },
    get contentText() {
      return contentText;
    },
    set contentText(v) {
      contentText = v;
    },
    get transcriptRenderMode() {
      return transcriptRenderMode;
    },
    set transcriptRenderMode(v) {
      transcriptRenderMode = v;
    },
    get editing() {
      return editing;
    },
    set editing(v) {
      editing = v;
    },
    get draft() {
      return draft;
    },
    set draft(v) {
      draft = v;
    },
    get draftTranscriptRenderMode() {
      return draftTranscriptRenderMode;
    },
    set draftTranscriptRenderMode(v) {
      draftTranscriptRenderMode = v;
    },
    get videoInfo() {
      return videoInfo;
    },
    set videoInfo(v) {
      videoInfo = v;
    },
    get summaryQualityScore() {
      return summaryQualityScore;
    },
    get summaryQualityNote() {
      return summaryQualityNote;
    },
    get summaryModelUsed() {
      return summaryModelUsed;
    },
    get summaryQualityModelUsed() {
      return summaryQualityModelUsed;
    },

    get formattingContent() {
      return formattingContent;
    },
    get formattingVideoId() {
      return formattingVideoId;
    },
    get regeneratingSummaryVideoIds() {
      return regeneratingSummaryVideoIds;
    },
    get revertingContent() {
      return revertingContent;
    },
    get revertingVideoId() {
      return revertingVideoId;
    },
    get resettingVideo() {
      return resettingVideo;
    },
    get resettingVideoId() {
      return resettingVideoId;
    },

    get formattingNotice() {
      return formattingNotice;
    },
    get formattingNoticeVideoId() {
      return formattingNoticeVideoId;
    },
    get formattingNoticeTone() {
      return formattingNoticeTone;
    },
    get formattingAttemptsUsed() {
      return formattingAttemptsUsed;
    },
    get formattingAttemptsMax() {
      return formattingAttemptsMax;
    },
    get formattingAttemptsVideoId() {
      return formattingAttemptsVideoId;
    },

    get originalTranscriptByVideoId() {
      return originalTranscriptByVideoId;
    },

    clear() {
      contentText = "";
      transcriptRenderMode = "plain_text";
      draft = "";
      draftTranscriptRenderMode = "plain_text";
      editing = false;
      videoInfo = null;
      resetSummaryQuality();
    },

    loadContent,
    resetSummaryQuality,
    clearFormattingFeedback,
    invalidateContentCache,
    applySummaryQuality,
    cacheLoadedSummary,

    startEdit() {
      editing = true;
      draft = contentText;
      draftTranscriptRenderMode = transcriptRenderMode;
    },

    cancelEdit() {
      editing = false;
      draft = contentText;
      draftTranscriptRenderMode = transcriptRenderMode;
    },

    async saveEdit() {
      const targetVideoId = options.getSelectedVideoId();
      if (!targetVideoId) return;
      if (contentMode === "info" || contentMode === "highlights") return;

      loadingContent = true;
      try {
        if (contentMode === "transcript") {
          const transcript = await updateTranscript(
            targetVideoId,
            draft,
            draftTranscriptRenderMode,
          );
          const presentation = resolveTranscriptPresentation(transcript);
          contentText = presentation.content;
          transcriptRenderMode = presentation.renderMode;
          draftTranscriptRenderMode = presentation.renderMode;
          invalidateContentCache(targetVideoId, "transcript");
          resetSummaryQuality();
          videoInfo = null;
        } else {
          const summary = await updateSummary(targetVideoId, draft);
          contentText = stripContentPrefix(
            summary.content || "Summary unavailable.",
          );
          invalidateContentCache(targetVideoId, "summary");
          applySummaryQuality(summary);
          const channelId = options.getSelectedChannelId();
          if (channelId && contentMode === "summary") {
            syncSummaryTrackingSession(summary, targetVideoId, channelId);
          }
          videoInfo = null;
        }
        editing = false;
      } finally {
        loadingContent = false;
      }
    },

    async regenerateSummaryContent() {
      const targetVideoId = options.getSelectedVideoId();
      if (!targetVideoId || contentMode !== "summary") return;

      regeneratingSummaryVideoIds = [
        ...regeneratingSummaryVideoIds.filter((id) => id !== targetVideoId),
        targetVideoId,
      ];

      options.setVideoStatus(targetVideoId, undefined, "loading");

      try {
        const summary = await apiRegenerateSummary(targetVideoId);
        invalidateContentCache(targetVideoId, "summary");
        options.setVideoStatus(targetVideoId, undefined, "ready");

        if (
          options.getSelectedVideoId() === targetVideoId &&
          contentMode === "summary"
        ) {
          contentText = stripContentPrefix(
            summary.content || "Summary unavailable.",
          );
          applySummaryQuality(summary);
          const channelId = options.getSelectedChannelId();
          if (channelId) {
            syncSummaryTrackingSession(summary, targetVideoId, channelId);
          }
          draft = contentText;
        }
      } catch (error) {
        options.setVideoStatus(targetVideoId, undefined, "failed");
        throw error;
      } finally {
        regeneratingSummaryVideoIds = regeneratingSummaryVideoIds.filter(
          (id) => id !== targetVideoId,
        );
      }
    },

    async resetVideoContent() {
      const targetVideoId = options.getSelectedVideoId();
      if (!targetVideoId) return;

      resettingVideo = true;
      resettingVideoId = targetVideoId;

      options.setVideoStatus(targetVideoId, "pending", "pending");
      invalidateContentCache(targetVideoId, "transcript");
      invalidateContentCache(targetVideoId, "summary");
      contentText = "";
      draft = "";

      try {
        const { resetVideo } = await import("$lib/api");
        await resetVideo(targetVideoId);
      } finally {
        resettingVideo = false;
        resettingVideoId = null;
      }
    },

    async cleanFormatting() {
      const targetVideoId = options.getSelectedVideoId();
      if (!targetVideoId || contentMode !== "transcript") return;
      const startedInEditMode = editing;
      const source = startedInEditMode ? draft : contentText;
      const requestId = ++formattingRequestSeq;

      activeFormattingRequest = requestId;
      formattingContent = true;
      formattingVideoId = targetVideoId;
      formattingNotice = "Formatting transcript with Ollama…";
      formattingNoticeVideoId = targetVideoId;
      formattingNoticeTone = "info";
      formattingAttemptsUsed = 0;
      formattingAttemptsMax = 5; // FORMAT_MAX_TURNS
      formattingAttemptsVideoId = targetVideoId;

      try {
        const result = await cleanTranscriptFormatting(targetVideoId, source);
        const attemptsSummary = buildFormattingAttemptSummary(result);
        formattingAttemptsUsed = result.attempts_used;
        formattingAttemptsMax = result.max_attempts;
        formattingAttemptsVideoId = targetVideoId;

        if (startedInEditMode) {
          if (
            activeFormattingRequest === requestId &&
            options.getSelectedVideoId() === targetVideoId &&
            editing
          ) {
            draft = result.content;
            if (result.content !== source) {
              draftTranscriptRenderMode = "markdown";
            }
          }
          formattingNotice =
            result.content === source
              ? `No formatting changes. ${attemptsSummary}`
              : `Formatting applied to draft. Save to persist. ${attemptsSummary}`;
          formattingNoticeVideoId = targetVideoId;
        } else {
          if (result.content !== source) {
            const transcript = await updateTranscript(
              targetVideoId,
              result.content,
              "markdown",
            );
            invalidateContentCache(targetVideoId, "transcript");
            if (
              activeFormattingRequest === requestId &&
              options.getSelectedVideoId() === targetVideoId &&
              !editing
            ) {
              const presentation = resolveTranscriptPresentation(transcript);
              contentText = presentation.content;
              transcriptRenderMode = presentation.renderMode;
              draftTranscriptRenderMode = presentation.renderMode;
              draft = contentText;
            }
          }
          formattingNotice =
            result.content === source
              ? `No formatting changes. ${attemptsSummary}`
              : `Formatting applied and saved. ${attemptsSummary}`;
          formattingNoticeVideoId = targetVideoId;
        }
        formattingNoticeTone = "success";
        if (result.timed_out) {
          formattingNotice = `Formatting reached the time limit. Current transcript was kept. ${attemptsSummary}`;
          formattingNoticeVideoId = targetVideoId;
          formattingNoticeTone = "warning";
        } else if (!result.preserved_text) {
          formattingNotice = `Safety guard kept original wording. Only spacing changes are allowed. ${attemptsSummary}`;
          formattingNoticeVideoId = targetVideoId;
          formattingNoticeTone = "warning";
        }
      } finally {
        if (activeFormattingRequest === requestId) {
          formattingContent = false;
          formattingVideoId = null;
        }
      }
    },

    async revertToOriginalTranscript() {
      const targetVideoId = options.getSelectedVideoId();
      if (!targetVideoId || contentMode !== "transcript") return;
      const original = originalTranscriptByVideoId[targetVideoId];
      if (!original) return;

      const startedInEditMode = editing;
      const source = startedInEditMode ? draft : contentText;
      if (source === original) {
        formattingNotice = "Already showing the original transcript.";
        formattingNoticeVideoId = targetVideoId;
        formattingNoticeTone = "info";
        formattingAttemptsUsed = null;
        formattingAttemptsMax = null;
        formattingAttemptsVideoId = null;
        return;
      }

      revertingContent = true;
      revertingVideoId = targetVideoId;
      formattingNotice = startedInEditMode
        ? "Reverting draft to original transcript…"
        : "Reverting transcript to original…";
      formattingNoticeVideoId = targetVideoId;
      formattingNoticeTone = "info";

      try {
        if (startedInEditMode) {
          if (options.getSelectedVideoId() === targetVideoId && editing) {
            draft = original;
            draftTranscriptRenderMode = "plain_text";
          }
          formattingNotice =
            "Draft reset to original transcript. Save to persist.";
        } else {
          const transcript = await updateTranscript(
            targetVideoId,
            original,
            "plain_text",
          );
          invalidateContentCache(targetVideoId, "transcript");
          if (options.getSelectedVideoId() === targetVideoId && !editing) {
            const presentation = resolveTranscriptPresentation(transcript);
            contentText = presentation.content;
            transcriptRenderMode = presentation.renderMode;
            draftTranscriptRenderMode = presentation.renderMode;
            draft = contentText;
          }
          formattingNotice = "Original transcript restored.";
        }
        formattingNoticeVideoId = targetVideoId;
        formattingNoticeTone = "success";
      } finally {
        revertingContent = false;
        revertingVideoId = null;
      }
    },
  };
}
