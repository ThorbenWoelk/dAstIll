import { createHighlight, deleteHighlight, getVideoHighlights } from "$lib/api";
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

export interface CreateHighlightStateOptions {
  getSelectedVideoId: () => string | null;
  getContentMode: () => string;
  onError: (message: string) => void;
}

export function createHighlightState(options: CreateHighlightStateOptions) {
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

  function getHighlightsForVideo(videoId: string): Highlight[] {
    return videoHighlightsByVideoId[videoId] ?? [];
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

  async function hydrateVideoHighlights(
    videoId: string,
    opts: { showError?: boolean } = {},
  ): Promise<Highlight[] | null> {
    try {
      const highlights = await getVideoHighlights(videoId);
      storeVideoHighlights(videoId, highlights);
      return highlights;
    } catch (error) {
      if (opts.showError) {
        options.onError((error as Error).message);
      }
      return null;
    }
  }

  async function saveSelectionHighlight(
    payload: CreateHighlightRequest,
  ): Promise<void> {
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
    } catch (error) {
      removeVideoHighlight(targetVideoId, optimisticHighlight.id);
      options.onError((error as Error).message);
    } finally {
      creatingHighlight = false;
      creatingHighlightVideoId = null;
    }
  }

  async function deleteExistingHighlight(highlightId: number): Promise<void> {
    const selectedVideoId = options.getSelectedVideoId();
    const targetVideoId =
      selectedVideoId ??
      Object.keys(videoHighlightsByVideoId).find((videoId) =>
        (videoHighlightsByVideoId[videoId] ?? []).some(
          (h) => Number(h.id) === highlightId,
        ),
      );
    if (!targetVideoId) {
      return;
    }
    deletingHighlightId = highlightId;

    try {
      await deleteHighlight(highlightId);
      removeVideoHighlight(targetVideoId, highlightId);
    } catch (error) {
      options.onError((error as Error).message);
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
    hydrateVideoHighlights,
    saveSelectionHighlight,
    deleteExistingHighlight,
    storeVideoHighlights,
    getHighlightsForVideo,
    hasHighlightsForVideo,
  };
}
