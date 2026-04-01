import { authState } from "$lib/auth-state.svelte";
import { createHighlight, getVideoHighlights } from "$lib/api";
import { presentAuthRequiredNoticeIfNeeded } from "$lib/auth-required-notice";
import { track } from "$lib/analytics/tracker";
import type {
  CreateHighlightRequest,
  Highlight,
  HighlightSource,
} from "$lib/types";
import {
  buildOptimisticHighlight,
  reconcileOptimisticHighlight,
} from "$lib/utils/highlights";
import {
  mergeVideoHighlights,
  removeVideoHighlightFromState,
} from "$lib/workspace/highlight-actions";
import {
  loadSessionHighlightsMap,
  resolveHighlightsScopeKey,
  saveSessionHighlightsMap,
  shouldUseSessionHighlights,
} from "$lib/workspace/session-highlights";
import type { WorkspaceContentMode } from "$lib/workspace/types";

export function createHomeWorkspaceHighlightController(options: {
  getSelectedVideoId: () => string | null;
  getSelectedChannelId: () => string | null;
  getContentMode: () => WorkspaceContentMode;
  getCanManageLibrary: () => boolean;
  onError: (message: string | null) => void;
}) {
  let videoHighlightsByVideoId = $state<Record<string, Highlight[]>>({});
  let nextOptimisticHighlightId = -1;
  let creatingHighlight = $state(false);
  let creatingHighlightVideoId = $state<string | null>(null);
  let deletingHighlightId = $state<number | null>(null);

  function storeVideoHighlights(videoId: string, highlights: Highlight[]) {
    videoHighlightsByVideoId = {
      ...videoHighlightsByVideoId,
      [videoId]: highlights,
    };
  }

  function hasHighlightsForVideo(videoId: string): boolean {
    return videoHighlightsByVideoId[videoId] !== undefined;
  }

  function mergeVideoHighlight(videoId: string, highlight: Highlight) {
    videoHighlightsByVideoId = mergeVideoHighlights(
      videoHighlightsByVideoId,
      videoId,
      highlight,
    );
  }

  function removeVideoHighlight(videoId: string, highlightId: number) {
    videoHighlightsByVideoId = removeVideoHighlightFromState(
      videoHighlightsByVideoId,
      videoId,
      highlightId,
    );
  }

  function persistSessionHighlightsIfNeeded() {
    if (!shouldUseSessionHighlights(authState.current)) return;
    const scope = resolveHighlightsScopeKey(authState.current);
    const existing = loadSessionHighlightsMap(scope);
    const next: Record<string, Highlight[]> = { ...existing };
    for (const [videoId, list] of Object.entries(videoHighlightsByVideoId)) {
      next[videoId] = list;
    }
    saveSessionHighlightsMap(scope, next);
  }

  async function hydrateVideoHighlights(
    videoId: string,
    opts: { showError?: boolean } = {},
  ): Promise<Highlight[] | null> {
    if (shouldUseSessionHighlights(authState.current)) {
      const scope = resolveHighlightsScopeKey(authState.current);
      const map = loadSessionHighlightsMap(scope);
      const highlights = map[videoId] ?? [];
      storeVideoHighlights(videoId, highlights);
      return highlights;
    }
    try {
      const highlights = await getVideoHighlights(videoId);
      storeVideoHighlights(videoId, highlights);
      return highlights;
    } catch (error) {
      if (opts.showError && !presentAuthRequiredNoticeIfNeeded(error)) {
        options.onError((error as Error).message);
      }
      return null;
    }
  }

  async function saveSelectionHighlight(payload: CreateHighlightRequest) {
    const selectedVideoId = options.getSelectedVideoId();
    const contentMode = options.getContentMode();

    if (
      !selectedVideoId ||
      (contentMode !== "transcript" && contentMode !== "summary")
    ) {
      return;
    }

    const targetVideoId = selectedVideoId;
    const optimisticHighlight = buildOptimisticHighlight(
      targetVideoId,
      payload,
      nextOptimisticHighlightId,
    );
    nextOptimisticHighlightId -= 1;

    mergeVideoHighlight(targetVideoId, optimisticHighlight);
    creatingHighlight = true;
    creatingHighlightVideoId = targetVideoId;
    options.onError(null);

    if (shouldUseSessionHighlights(authState.current)) {
      try {
        persistSessionHighlightsIfNeeded();
        const channelId = options.getSelectedChannelId();
        if (channelId) {
          track({
            event: "highlight_created",
            video_id: targetVideoId,
            channel_id: channelId,
            source: payload.source as HighlightSource,
          });
        }
      } finally {
        creatingHighlight = false;
        creatingHighlightVideoId = null;
      }
      return;
    }

    if (!options.getCanManageLibrary()) {
      removeVideoHighlight(targetVideoId, optimisticHighlight.id);
      creatingHighlight = false;
      creatingHighlightVideoId = null;
      return;
    }

    try {
      const highlight = await createHighlight(targetVideoId, payload);
      storeVideoHighlights(
        targetVideoId,
        reconcileOptimisticHighlight(
          videoHighlightsByVideoId[targetVideoId] ?? [],
          optimisticHighlight.id,
          highlight,
        ),
      );
      const channelId = options.getSelectedChannelId();
      if (channelId) {
        track({
          event: "highlight_created",
          video_id: targetVideoId,
          channel_id: channelId,
          source: payload.source as HighlightSource,
        });
      }
    } catch (error) {
      removeVideoHighlight(targetVideoId, optimisticHighlight.id);
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        options.onError((error as Error).message);
      }
    } finally {
      creatingHighlight = false;
      creatingHighlightVideoId = null;
    }
  }

  async function deleteExistingHighlight(highlightId: number) {
    const targetVideoId =
      options.getSelectedVideoId() ??
      Object.keys(videoHighlightsByVideoId).find((videoId) =>
        (videoHighlightsByVideoId[videoId] ?? []).some(
          (highlight) => Number(highlight.id) === highlightId,
        ),
      );
    if (!targetVideoId) {
      return;
    }
    deletingHighlightId = highlightId;
    options.onError(null);

    if (shouldUseSessionHighlights(authState.current)) {
      try {
        removeVideoHighlight(targetVideoId, highlightId);
        persistSessionHighlightsIfNeeded();
      } finally {
        deletingHighlightId = null;
      }
      return;
    }

    if (!options.getCanManageLibrary()) {
      deletingHighlightId = null;
      return;
    }

    try {
      const { deleteHighlight } = await import("$lib/api");
      await deleteHighlight(highlightId);
      removeVideoHighlight(targetVideoId, highlightId);
    } catch (error) {
      if (!presentAuthRequiredNoticeIfNeeded(error)) {
        options.onError((error as Error).message);
      }
    } finally {
      deletingHighlightId = null;
    }
  }

  return {
    get videoHighlightsByVideoId() {
      return videoHighlightsByVideoId;
    },
    get creatingHighlight() {
      return creatingHighlight;
    },
    get creatingHighlightVideoId() {
      return creatingHighlightVideoId;
    },
    get deletingHighlightId() {
      return deletingHighlightId;
    },
    hasHighlightsForVideo,
    hydrateVideoHighlights,
    saveSelectionHighlight,
    deleteExistingHighlight,
  };
}
