/**
 * While the reader scrolls down a page, the tab bar and the section bar
 * tuck away so the story has the screen. Scrolling back up, or reaching the
 * top or the end of the page, brings them back. CSS decides which bars
 * tuck at which size; this module only follows the scroll.
 */

/** Above this scroll position the bars always show. */
const ALWAYS_SHOWN_ABOVE_PX = 240;
/** Scroll this far one way before the bars change. */
const TURN_AFTER_PX = 16;

export interface ScrollFollow {
  tucked: boolean;
  lastY: number;
  /** Distance scrolled in the current direction; negative is up. */
  travel: number;
}

export function followScroll(
  state: ScrollFollow,
  y: number,
  viewportHeight: number,
  pageHeight: number,
): ScrollFollow {
  if (y <= ALWAYS_SHOWN_ABOVE_PX || y + viewportHeight >= pageHeight - 1) {
    return { tucked: false, lastY: y, travel: 0 };
  }
  const delta = y - state.lastY;
  if (delta === 0) return state;
  // A jump (restored scroll position, a new page) is not the reader scrolling.
  if (Math.abs(delta) >= viewportHeight) {
    return { ...state, lastY: y, travel: 0 };
  }
  const travel =
    Math.sign(delta) === Math.sign(state.travel) ? state.travel + delta : delta;
  let tucked = state.tucked;
  if (travel >= TURN_AFTER_PX) tucked = true;
  else if (travel <= -TURN_AFTER_PX) tucked = false;
  return { tucked, lastY: y, travel };
}

export interface ScrollWatch {
  /** Show the bars again, for example after moving to another page. */
  reset(): void;
  stop(): void;
}

/** Calls `onChange` whenever the bars should tuck away or come back. */
export function watchScroll(onChange: (tucked: boolean) => void): ScrollWatch {
  let state: ScrollFollow = { tucked: false, lastY: window.scrollY, travel: 0 };
  let frame = 0;

  function update(next: ScrollFollow) {
    if (next.tucked !== state.tucked) onChange(next.tucked);
    state = next;
  }

  function readScroll() {
    frame = 0;
    update(
      followScroll(
        state,
        window.scrollY,
        window.innerHeight,
        document.documentElement.scrollHeight,
      ),
    );
  }

  function handleScroll() {
    if (!frame) frame = requestAnimationFrame(readScroll);
  }

  /** Keyboard focus landing in a tucked bar brings it back. */
  function handleFocus(event: FocusEvent) {
    if (event.target instanceof Element && event.target.closest("nav")) {
      update({ ...state, tucked: false, travel: 0 });
    }
  }

  window.addEventListener("scroll", handleScroll, { passive: true });
  document.addEventListener("focusin", handleFocus);

  return {
    reset() {
      update({ tucked: false, lastY: window.scrollY, travel: 0 });
    },
    stop() {
      cancelAnimationFrame(frame);
      window.removeEventListener("scroll", handleScroll);
      document.removeEventListener("focusin", handleFocus);
    },
  };
}
