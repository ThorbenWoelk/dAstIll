import { fetchChannelSummaries, setStoryRead, type Channel } from "$lib/api";
import {
  loadStoredEdition,
  loadStoredSection,
  storeEdition,
  storeSection,
} from "$lib/edition/keepsake";
import { printEdition } from "$lib/edition/printing";
import {
  chooseLeadStory,
  FRONT_PAGE,
  insertStory,
  listSections,
  locateStory,
  storiesInSection,
  storyAfterFinishing,
  type SectionId,
  type Story,
  type TurnDirection,
} from "$lib/edition/stories";

export type EditionStatus = "loading" | "ready" | "failed";

/** Reload when the tab returns after this long. No polling otherwise. */
const STALE_AFTER_MS = 30 * 60 * 1000;
const RAIL_LENGTH = 6;

function messageOf(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/**
 * State for one reader's paper. Methods are the only write path; components
 * read the fields and derived values.
 */
export class EditionReader {
  status = $state<EditionStatus>("loading");
  refreshing = $state(false);
  channels = $state.raw<Channel[]>([]);
  stories = $state.raw<Story[]>([]);
  section = $state<SectionId>(FRONT_PAGE);
  pickedId = $state<string | null>(null);
  lastRead = $state.raw<Story | null>(null);
  /** Something went wrong but the paper is still readable. */
  notice = $state<string | null>(null);
  /** Nothing to show because loading failed. */
  failure = $state<string | null>(null);

  visible = $derived(storiesInSection(this.stories, this.section));
  lead = $derived(chooseLeadStory(this.visible, this.pickedId));
  position = $derived(locateStory(this.visible, this.lead?.id ?? null));
  alsoInEdition = $derived(
    this.visible
      .filter((story) => story.id !== this.lead?.id)
      .slice(0, RAIL_LENGTH),
  );
  sections = $derived(listSections(this.channels, this.stories));

  readonly #uid: string;
  #loadedAt = 0;
  #printRun = 0;
  /** Stories marked read whose request has not finished yet. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping, never rendered
  readonly #pendingRead = new Set<string>();
  /** Read-state writes per story, chained so undo lands after the read. */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping, never rendered
  readonly #writes = new Map<string, Promise<unknown>>();

  constructor(uid: string) {
    this.#uid = uid;
    const stored = loadStoredEdition(uid);
    if (stored) {
      this.channels = stored.channels;
      this.stories = stored.stories;
      this.status = "ready";
    }
    this.section = loadStoredSection(uid) ?? FRONT_PAGE;
  }

  async refresh() {
    const run = ++this.#printRun;
    this.refreshing = true;
    if (this.stories.length === 0 && this.channels.length === 0) {
      this.status = "loading";
    }
    try {
      const edition = await printEdition(fetchChannelSummaries, (partial) => {
        // First load only: show stories as channels arrive. A stored edition
        // already on screen stays until the full edition replaces it.
        if (run !== this.#printRun || this.status !== "loading") return;
        if (partial.stories.length === 0) return;
        this.channels = partial.channels;
        this.stories = partial.stories.filter(
          (story) => !this.#pendingRead.has(story.id),
        );
        this.status = "ready";
      });
      if (run !== this.#printRun) return;
      this.channels = edition.channels;
      this.stories = edition.stories.filter(
        (story) => !this.#pendingRead.has(story.id),
      );
      if (!this.channels.some((channel) => channel.id === this.section)) {
        this.section = FRONT_PAGE;
      }
      this.notice =
        edition.missingChannelIds.length > 0
          ? "Some sections could not be loaded. Refresh to try again."
          : null;
      this.failure = null;
      this.status = "ready";
      this.#loadedAt = Date.now();
      this.#remember();
    } catch (cause) {
      if (run !== this.#printRun) return;
      if (this.status === "ready") {
        this.notice = `Could not refresh: ${messageOf(cause)}`;
      } else {
        this.failure = messageOf(cause);
        this.status = "failed";
      }
    } finally {
      if (run === this.#printRun) this.refreshing = false;
    }
  }

  refreshIfStale() {
    if (!this.refreshing && Date.now() - this.#loadedAt > STALE_AFTER_MS) {
      void this.refresh();
    }
  }

  showSection(section: SectionId) {
    this.section = section;
    this.pickedId = null;
    storeSection(this.#uid, section);
  }

  /** Read a story from "Also in this edition" next. */
  pick(storyId: string) {
    this.pickedId = storyId;
  }

  canTurn(direction: TurnDirection): boolean {
    return this.position[direction] !== null;
  }

  /** Lead with the story after or before this one. Nothing is marked read. */
  turn(direction: TurnDirection): boolean {
    const story = this.position[direction];
    if (!story) return false;
    this.pickedId = story.id;
    return true;
  }

  async markLeadRead() {
    const story = this.lead;
    if (!story) return;
    const following = storyAfterFinishing(this.visible, story.id);
    this.stories = this.stories.filter((s) => s.id !== story.id);
    this.lastRead = story;
    this.pickedId = following;
    this.#pendingRead.add(story.id);
    this.#remember();
    try {
      await this.#writeReadState(story.id, true);
    } catch (cause) {
      this.stories = insertStory(this.stories, story);
      if (this.lastRead?.id === story.id) this.lastRead = null;
      // Still on the story that took its place: put it back on the page.
      if (this.pickedId === following) this.pickedId = story.id;
      this.notice = `Could not mark as read: ${messageOf(cause)}`;
      this.#remember();
    } finally {
      this.#pendingRead.delete(story.id);
    }
  }

  async undoLastRead() {
    const story = this.lastRead;
    if (!story) return;
    this.lastRead = null;
    this.stories = insertStory(this.stories, story);
    this.pickedId = story.id;
    this.#pendingRead.delete(story.id);
    this.#remember();
    try {
      await this.#writeReadState(story.id, false);
    } catch (cause) {
      this.stories = this.stories.filter((s) => s.id !== story.id);
      this.notice = `Could not undo: ${messageOf(cause)}`;
      this.#remember();
    }
  }

  dismissNotice() {
    this.notice = null;
  }

  #writeReadState(storyId: string, read: boolean): Promise<unknown> {
    const previous = this.#writes.get(storyId) ?? Promise.resolve();
    const next = previous
      .catch(() => undefined)
      .then(() => setStoryRead(storyId, read));
    this.#writes.set(storyId, next);
    void next
      .catch(() => undefined)
      .finally(() => {
        if (this.#writes.get(storyId) === next) this.#writes.delete(storyId);
      });
    return next;
  }

  #remember() {
    storeEdition(this.#uid, this.channels, this.stories);
  }
}
