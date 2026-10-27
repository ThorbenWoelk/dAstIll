import {
  createHighlight,
  createWritingFlag,
  deleteHighlight,
  deleteWritingFlag,
  listStoryHighlights,
  listStoryWritingFlags,
  type CreateHighlightRequest,
  type CreateWritingFlagRequest,
  type Highlight,
  type WritingFlag,
} from "$lib/api";

const PENDING_PREFIX = "pending-";

function messageOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

interface SavedNote {
  id: string;
  video_id: string;
  created_at: string;
}

interface NoteRequests<Note, Draft> {
  list: (videoId: string) => Promise<Note[]>;
  create: (videoId: string, draft: Draft) => Promise<Note>;
  remove: (id: string) => Promise<void>;
  /** "highlight", "flag": used in error messages. */
  noun: string;
}

/**
 * Notes the reader keeps on the story on screen: highlights or writing
 * flags. Adding and removing show at once and roll back if the server
 * refuses.
 */
class StoryNotes<Note extends SavedNote, Draft extends object> {
  items = $state.raw<Note[]>([]);
  error = $state<string | null>(null);

  readonly #requests: NoteRequests<Note, Draft>;
  #videoId: string | null = null;
  #pendingCount = 0;
  /** Notes of stories already shown, so going back is instant. */
  readonly #byVideo = new Map<string, Note[]>();

  constructor(requests: NoteRequests<Note, Draft>) {
    this.#requests = requests;
  }

  async show(videoId: string) {
    this.#videoId = videoId;
    this.items = this.#byVideo.get(videoId) ?? [];
    this.error = null;
    try {
      const loaded = await this.#requests.list(videoId);
      const pending = (this.#byVideo.get(videoId) ?? []).filter((note) =>
        note.id.startsWith(PENDING_PREFIX),
      );
      this.#set(videoId, [...loaded, ...pending]);
    } catch (cause) {
      if (this.#videoId === videoId) {
        this.error = `Could not load ${this.#requests.noun}s: ${messageOf(cause)}`;
      }
    }
  }

  async add(videoId: string, draft: Draft) {
    const pending = {
      ...draft,
      id: `${PENDING_PREFIX}${++this.#pendingCount}`,
      video_id: videoId,
      created_at: new Date().toISOString(),
    } as unknown as Note;
    this.#change(videoId, (list) => [...list, pending]);
    try {
      const saved = await this.#requests.create(videoId, draft);
      this.#change(videoId, (list) => [
        ...list.filter(
          (note) => note.id !== pending.id && note.id !== saved.id,
        ),
        saved,
      ]);
    } catch (cause) {
      this.#change(videoId, (list) =>
        list.filter((note) => note.id !== pending.id),
      );
      this.error = `Could not save the ${this.#requests.noun}: ${messageOf(cause)}`;
    }
  }

  async remove(videoId: string, id: string) {
    if (id.startsWith(PENDING_PREFIX)) return;
    const removed = (this.#byVideo.get(videoId) ?? []).find(
      (note) => note.id === id,
    );
    if (!removed) return;
    this.#change(videoId, (list) => list.filter((note) => note.id !== id));
    try {
      await this.#requests.remove(id);
    } catch (cause) {
      this.#change(videoId, (list) => [...list, removed]);
      this.error = `Could not remove the ${this.#requests.noun}: ${messageOf(cause)}`;
    }
  }

  dismissError() {
    this.error = null;
  }

  #change(videoId: string, update: (list: Note[]) => Note[]) {
    this.#set(videoId, update(this.#byVideo.get(videoId) ?? []));
  }

  #set(videoId: string, list: Note[]) {
    this.#byVideo.set(videoId, list);
    if (this.#videoId === videoId) this.items = list;
  }
}

/** Highlights of the story on screen. */
export class StoryHighlights extends StoryNotes<
  Highlight,
  CreateHighlightRequest
> {
  constructor() {
    super({
      list: listStoryHighlights,
      create: createHighlight,
      remove: deleteHighlight,
      noun: "highlight",
    });
  }
}

/** Passages of the story on screen the reader flagged as bad writing. */
export class StoryWritingFlags extends StoryNotes<
  WritingFlag,
  CreateWritingFlagRequest
> {
  constructor() {
    super({
      list: listStoryWritingFlags,
      create: createWritingFlag,
      remove: deleteWritingFlag,
      noun: "flag",
    });
  }
}
