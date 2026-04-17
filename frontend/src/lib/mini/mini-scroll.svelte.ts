import type { MiniReaderState } from "./mini-reader-state.svelte";

const SCROLL_COLLAPSE_THRESHOLD = 32;
const SCROLL_EXPAND_THRESHOLD = 4;

export interface MiniScrollController {
  readonly scrolledFromTop: boolean;
  bind(element: HTMLElement | null): void;
  onScroll(): void;
  reset(): void;
}

export function createMiniScrollController(
  mini: MiniReaderState,
): MiniScrollController {
  let container = $state<HTMLElement | null>(null);
  let scrolledFromTop = $state(false);

  function onScroll() {
    if (!container) return;
    const { scrollTop, scrollHeight, clientHeight } = container;
    mini.updateReadProgress(scrollTop, scrollHeight, clientHeight);
    if (!scrolledFromTop && scrollTop > SCROLL_COLLAPSE_THRESHOLD) {
      scrolledFromTop = true;
    } else if (scrolledFromTop && scrollTop < SCROLL_EXPAND_THRESHOLD) {
      scrolledFromTop = false;
    }
  }

  function reset() {
    scrolledFromTop = false;
    container?.scrollTo({ top: 0, behavior: "instant" });
  }

  return {
    get scrolledFromTop() {
      return scrolledFromTop;
    },
    bind(element) {
      container = element;
    },
    onScroll,
    reset,
  };
}
