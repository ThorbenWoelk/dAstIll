import {
  createHighlight,
  deleteHighlight,
  listStoryHighlights,
  type CreateHighlightRequest,
  type Highlight,
} from "$lib/api";

const PENDING_PREFIX = "pending-";

function messageOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/**
 * Highlights of the story on screen. Adding and removing show at once and
 * roll back if the server refuses.
 */
export class StoryHighlights {
  items = $state.raw<Highlight[]>([]);
  error = $state<string | null>(null);

  #videoId: string | null = null;
  #pendingCount = 0;
  /** Highlights of stories already shown, so going back is instant. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping, never rendered
  readonly #byVideo = new Map<string, Highlight[]>();

  async show(videoId: string) {
    this.#videoId = videoId;
    this.items = this.#byVideo.get(videoId) ?? [];
    this.error = null;
    try {
      const loaded = await listStoryHighlights(videoId);
      const pending = (this.#byVideo.get(videoId) ?? []).filter((h) =>
        h.id.startsWith(PENDING_PREFIX),
      );
      this.#set(videoId, [...loaded, ...pending]);
    } catch (cause) {
      if (this.#videoId === videoId) {
        this.error = `Could not load highlights: ${messageOf(cause)}`;
      }
    }
  }

  async add(videoId: string, draft: CreateHighlightRequest) {
    const pending: Highlight = {
      ...draft,
      id: `${PENDING_PREFIX}${++this.#pendingCount}`,
      video_id: videoId,
      // eslint-disable-next-line svelte/prefer-svelte-reactivity -- a timestamp, never mutated
      created_at: new Date().toISOString(),
    };
    this.#change(videoId, (list) => [...list, pending]);
    try {
      const saved = await createHighlight(videoId, draft);
      this.#change(videoId, (list) => [
        ...list.filter((h) => h.id !== pending.id && h.id !== saved.id),
        saved,
      ]);
    } catch (cause) {
      this.#change(videoId, (list) => list.filter((h) => h.id !== pending.id));
      this.error = `Could not save the highlight: ${messageOf(cause)}`;
    }
  }

  async remove(videoId: string, highlightId: string) {
    if (highlightId.startsWith(PENDING_PREFIX)) return;
    const removed = (this.#byVideo.get(videoId) ?? []).find(
      (h) => h.id === highlightId,
    );
    if (!removed) return;
    this.#change(videoId, (list) => list.filter((h) => h.id !== highlightId));
    try {
      await deleteHighlight(highlightId);
    } catch (cause) {
      this.#change(videoId, (list) => [...list, removed]);
      this.error = `Could not remove the highlight: ${messageOf(cause)}`;
    }
  }

  dismissError() {
    this.error = null;
  }

  #change(videoId: string, update: (list: Highlight[]) => Highlight[]) {
    this.#set(videoId, update(this.#byVideo.get(videoId) ?? []));
  }

  #set(videoId: string, list: Highlight[]) {
    this.#byVideo.set(videoId, list);
    if (this.#videoId === videoId) this.items = list;
  }
}
