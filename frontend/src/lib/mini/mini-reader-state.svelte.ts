import { getMiniReader, updateMiniReadStatus } from "$lib/api";
import { authState } from "$lib/auth-state.svelte";
import type { CreateHighlightRequest, Highlight } from "$lib/types";
import type { MiniReader, MiniSummaryItem } from "$lib/transport-types";
import { renderMarkdown } from "$lib/utils/markdown";
import { createHomeWorkspaceHighlightController } from "$lib/workspace/home-workspace-highlight-controller.svelte";

export const MINI_DEFAULT_SHOW_UNREAD_ONLY = true;

export function chooseActiveVideoId(
  summaries: MiniSummaryItem[],
  preferredVideoId?: string | null,
): string | null {
  if (preferredVideoId) {
    const match = summaries.find((s) => s.video_id === preferredVideoId);
    if (match) return match.video_id;
  }
  const firstUnread = summaries.find((s) => !s.read);
  return firstUnread?.video_id ?? summaries[0]?.video_id ?? null;
}

export function findNextUnreadVideoId(
  summaries: MiniSummaryItem[],
  currentVideoId: string,
): string | null {
  if (summaries.length === 0) return null;

  const currentIndex = summaries.findIndex(
    (s) => s.video_id === currentVideoId,
  );
  const startIndex = currentIndex >= 0 ? currentIndex : -1;

  for (let offset = 1; offset <= summaries.length; offset += 1) {
    const summary = summaries[(startIndex + offset) % summaries.length];
    if (summary && !summary.read) {
      return summary.video_id;
    }
  }

  return null;
}

export function selectMiniSummaryHighlights(
  videoId: string | null | undefined,
  highlightsByVideoId: Record<string, Highlight[]>,
): Highlight[] {
  if (!videoId) return [];
  return (highlightsByVideoId[videoId] ?? []).filter(
    (highlight) => highlight.source === "summary",
  );
}

export class MiniReaderState {
  reader = $state<MiniReader | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedChannelId = $state<string | null>(null);
  activeVideoId = $state<string | null>(null);
  showUnreadOnly = $state(MINI_DEFAULT_SHOW_UNREAD_ONLY);
  markingRead = $state(false);
  contentKey = $state(0);
  readProgress = $state(0);
  highlightController = createHomeWorkspaceHighlightController({
    getSelectedVideoId: () => this.activeSummary?.video_id ?? null,
    getSelectedChannelId: () =>
      this.activeSummary?.channel_id ?? this.selectedChannelId,
    getContentMode: () => "summary",
    getCanManageLibrary: () => authState.current.authState === "authenticated",
    onError: (message) => {
      this.error = message;
    },
  });

  visibleSummaries = $derived(
    this.reader
      ? this.showUnreadOnly
        ? this.reader.summaries.filter((s) => !s.read)
        : this.reader.summaries
      : [],
  );

  activeIndex = $derived(
    this.visibleSummaries.findIndex((s) => s.video_id === this.activeVideoId),
  );

  activeSummary = $derived(
    this.activeIndex >= 0
      ? this.visibleSummaries[this.activeIndex]
      : (this.visibleSummaries[0] ?? null),
  );

  activeSummaryHtml = $derived(
    this.activeSummary
      ? renderMarkdown(this.activeSummary.summary_content)
      : "",
  );
  activeSummaryHighlights = $derived(
    selectMiniSummaryHighlights(
      this.activeSummary?.video_id ?? null,
      this.highlightController.videoHighlightsByVideoId,
    ),
  );
  creatingHighlight = $derived(this.highlightController.creatingHighlight);
  creatingHighlightVideoId = $derived(
    this.highlightController.creatingHighlightVideoId,
  );
  deletingHighlightId = $derived(this.highlightController.deletingHighlightId);

  canGoPrev = $derived(this.activeIndex > 0);
  canGoNext = $derived(
    this.activeIndex >= 0 &&
      this.activeIndex < this.visibleSummaries.length - 1,
  );
  unreadCount = $derived(
    this.reader?.summaries.filter((s) => !s.read).length ?? 0,
  );
  activeFilterCount = $derived(this.showUnreadOnly ? 1 : 0);

  async loadReader(
    channelId?: string | null,
    preferredVideoId?: string | null,
  ) {
    this.loading = true;
    this.error = null;
    try {
      const next = await getMiniReader(channelId);
      this.reader = next;
      this.selectedChannelId = next.selected_channel_id ?? null;
      this.activeVideoId = chooseActiveVideoId(
        next.summaries,
        preferredVideoId,
      );
      this.readProgress = 0;
    } catch (cause) {
      this.reader = null;
      this.selectedChannelId = null;
      this.activeVideoId = null;
      this.error =
        cause instanceof Error ? cause.message : "Could not load dastill-mini.";
    } finally {
      this.loading = false;
    }
  }

  stepSummary(delta: -1 | 1) {
    if (!this.activeSummary) return;
    const nextIndex = this.activeIndex + delta;
    const nextSummary = this.visibleSummaries[nextIndex];
    if (!nextSummary) return;
    this.activeVideoId = nextSummary.video_id;
    this.contentKey += 1;
    this.readProgress = 0;
  }

  jumpToSummary(videoId: string) {
    this.activeVideoId = videoId;
    this.contentKey += 1;
    this.readProgress = 0;
  }

  async markActiveSummaryRead() {
    if (!this.activeSummary || this.markingRead) return;
    this.markingRead = true;
    this.error = null;
    try {
      await updateMiniReadStatus(this.activeSummary.video_id, true);
      if (!this.reader) return;
      const markedId = this.activeSummary.video_id;
      this.reader = {
        ...this.reader,
        summaries: this.reader.summaries.map((s) =>
          s.video_id === markedId ? { ...s, read: true } : s,
        ),
      };
      const nextVisible = this.showUnreadOnly
        ? this.reader.summaries.filter(
            (s) => s.video_id !== markedId && !s.read,
          )
        : this.reader.summaries.map((s) =>
            s.video_id === markedId ? { ...s, read: true } : s,
          );
      this.activeVideoId = chooseActiveVideoId(nextVisible, markedId);
      this.contentKey += 1;
      this.readProgress = 0;
    } catch (cause) {
      this.error =
        cause instanceof Error
          ? cause.message
          : "Could not update read status.";
    } finally {
      this.markingRead = false;
    }
  }

  async markActiveSummaryReadAndAdvance() {
    if (!this.activeSummary || this.markingRead) return;
    this.markingRead = true;
    this.error = null;
    try {
      const markedId = this.activeSummary.video_id;
      await updateMiniReadStatus(markedId, true);
      if (!this.reader) return;

      const summaries = this.reader.summaries.map((s) =>
        s.video_id === markedId ? { ...s, read: true } : s,
      );
      this.reader = {
        ...this.reader,
        summaries,
      };

      const nextUnreadVideoId = findNextUnreadVideoId(summaries, markedId);
      this.activeVideoId =
        nextUnreadVideoId ?? (this.showUnreadOnly ? null : markedId);
      this.contentKey += 1;
      this.readProgress = 0;
    } catch (cause) {
      this.error =
        cause instanceof Error
          ? cause.message
          : "Could not update read status.";
    } finally {
      this.markingRead = false;
    }
  }

  async selectChannel(channelId: string) {
    if (!channelId || channelId === this.selectedChannelId) return;
    this.selectedChannelId = channelId;
    await this.loadReader(channelId);
  }

  hydrateActiveSummaryHighlights() {
    const videoId = this.activeSummary?.video_id;
    if (!videoId || this.highlightController.hasHighlightsForVideo(videoId))
      return;
    void this.highlightController.hydrateVideoHighlights(videoId, {
      showError: true,
    });
  }

  saveSelectionHighlight(payload: CreateHighlightRequest) {
    return this.highlightController.saveSelectionHighlight(payload);
  }

  deleteExistingHighlight(highlightId: number) {
    return this.highlightController.deleteExistingHighlight(highlightId);
  }

  toggleUnreadFilter() {
    this.showUnreadOnly = !this.showUnreadOnly;
  }

  clearUnreadFilter() {
    this.showUnreadOnly = false;
  }

  updateReadProgress(
    scrollTop: number,
    scrollHeight: number,
    clientHeight: number,
  ) {
    const maxScroll = scrollHeight - clientHeight;
    this.readProgress = maxScroll > 0 ? Math.min(1, scrollTop / maxScroll) : 0;
  }

  reconcileActiveVideo() {
    const nextId = chooseActiveVideoId(
      this.visibleSummaries,
      this.activeVideoId,
    );
    if (nextId !== this.activeVideoId) {
      this.activeVideoId = nextId;
      this.contentKey += 1;
      this.readProgress = 0;
    }
  }
}

export function createMiniReaderState(): MiniReaderState {
  return new MiniReaderState();
}
