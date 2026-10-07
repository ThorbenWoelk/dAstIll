import type { Attachment } from "svelte/attachments";
import type { TurnDirection } from "$lib/edition/stories";
import {
  directionOf,
  followFinger,
  readGestureAxis,
  settleSwipe,
  swipeVelocity,
  type GestureAxis,
  type TouchSample,
} from "$lib/navigation/swipe";

export interface StorySwipeOptions {
  /** Whether there is a story that way. Asked while the finger moves. */
  canGo: (direction: TurnDirection) => boolean;
  /** Called once the story has slid out of view. */
  go: (direction: TurnDirection) => void;
}

/** Touches this close to a screen edge belong to the system back gesture. */
const SCREEN_EDGE_PX = 24;
const LEAVE_MS = 180;
const SETTLE_MS = 220;
/** The click a swipe can trigger arrives right after the finger lifts. */
const SWALLOW_CLICK_MS = 100;

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** Code blocks and wide tables scroll sideways; leave their swipes alone. */
function startsInSideScroller(target: EventTarget | null, root: Element) {
  for (
    let element = target instanceof Element ? target : null;
    element && element !== root;
    element = element.parentElement
  ) {
    if (element.scrollWidth <= element.clientWidth) continue;
    const overflow = getComputedStyle(element).overflowX;
    if (overflow === "auto" || overflow === "scroll") return true;
  }
  return false;
}

/**
 * Lets a touch reader swipe the story sideways: it follows the finger, and
 * a long or quick swipe slides it out and turns to the next or previous
 * story. Vertical scrolling and pinch zoom stay with the browser. Mouse
 * drags keep selecting text.
 */
export function storySwipe(
  options: StorySwipeOptions,
): Attachment<HTMLElement> {
  return (node) => {
    let pointerId: number | null = null;
    let axis: GestureAxis = "undecided";
    let startX = 0;
    let startY = 0;
    let dx = 0;
    let samples: TouchSample[] = [];
    let leaving = false;
    let swallowClickUntil = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;

    node.style.touchAction = "pan-y pinch-zoom";

    function place(offset: number, opacity: number, transition = "none") {
      node.style.transition = transition;
      node.style.transform = offset ? `translateX(${offset}px)` : "";
      node.style.opacity = opacity < 1 ? String(opacity) : "";
    }

    function settleBack() {
      if (prefersReducedMotion()) return place(0, 1);
      place(
        0,
        1,
        `transform ${SETTLE_MS}ms ease-out, opacity ${SETTLE_MS}ms ease-out`,
      );
      timer = setTimeout(() => place(0, 1), SETTLE_MS);
    }

    function leave(direction: TurnDirection) {
      const finish = () => {
        leaving = false;
        place(0, 1);
        options.go(direction);
      };
      if (prefersReducedMotion()) return finish();
      leaving = true;
      const away = (direction === "next" ? -1 : 1) * node.offsetWidth;
      place(
        away,
        0,
        `transform ${LEAVE_MS}ms ease-in, opacity ${LEAVE_MS}ms ease-in`,
      );
      timer = setTimeout(finish, LEAVE_MS);
    }

    function handleDown(event: PointerEvent) {
      if (event.pointerType !== "touch" || !event.isPrimary) return;
      if (leaving || pointerId !== null) return;
      const edge = Math.min(event.clientX, window.innerWidth - event.clientX);
      if (edge < SCREEN_EDGE_PX) return;
      // A selection in progress belongs to the highlighter.
      if (document.getSelection()?.isCollapsed === false) return;
      if (startsInSideScroller(event.target, node)) return;
      pointerId = event.pointerId;
      axis = "undecided";
      startX = event.clientX;
      startY = event.clientY;
      dx = 0;
      samples = [{ x: event.clientX, time: event.timeStamp }];
    }

    function handleMove(event: PointerEvent) {
      if (event.pointerId !== pointerId) return;
      if (axis === "undecided") {
        // A long press that started selecting text is not a swipe.
        if (document.getSelection()?.isCollapsed === false) {
          pointerId = null;
          return;
        }
        axis = readGestureAxis(event.clientX - startX, event.clientY - startY);
        if (axis === "vertical") pointerId = null;
        if (axis !== "horizontal") return;
        clearTimeout(timer);
      }
      dx = event.clientX - startX;
      samples.push({ x: event.clientX, time: event.timeStamp });
      if (samples.length > 12) samples.shift();
      const offset = followFinger(dx, options.canGo(directionOf(dx)));
      const fade = Math.min(Math.abs(offset) / node.offsetWidth, 1) * 0.4;
      place(offset, 1 - fade);
    }

    function handleUp(event: PointerEvent) {
      if (event.pointerId !== pointerId) return;
      pointerId = null;
      if (axis !== "horizontal") return;
      swallowClickUntil = performance.now() + SWALLOW_CLICK_MS;
      samples.push({ x: event.clientX, time: event.timeStamp });
      const direction = settleSwipe(
        dx,
        swipeVelocity(samples),
        node.offsetWidth,
        options.canGo,
      );
      if (direction) leave(direction);
      else settleBack();
    }

    function handleCancel(event: PointerEvent) {
      if (event.pointerId !== pointerId) return;
      pointerId = null;
      if (axis === "horizontal") settleBack();
    }

    /** Keep the page still while the story moves sideways. */
    function holdScroll(event: TouchEvent) {
      if (pointerId !== null && axis === "horizontal" && event.cancelable) {
        event.preventDefault();
      }
    }

    function swallowClick(event: MouseEvent) {
      if (performance.now() < swallowClickUntil) {
        event.preventDefault();
        event.stopPropagation();
      }
    }

    node.addEventListener("pointerdown", handleDown);
    node.addEventListener("pointermove", handleMove);
    node.addEventListener("pointerup", handleUp);
    node.addEventListener("pointercancel", handleCancel);
    node.addEventListener("touchmove", holdScroll, { passive: false });
    node.addEventListener("click", swallowClick, true);

    return () => {
      clearTimeout(timer);
      node.removeEventListener("pointerdown", handleDown);
      node.removeEventListener("pointermove", handleMove);
      node.removeEventListener("pointerup", handleUp);
      node.removeEventListener("pointercancel", handleCancel);
      node.removeEventListener("touchmove", holdScroll);
      node.removeEventListener("click", swallowClick, true);
      place(0, 1);
      node.style.touchAction = "";
    };
  };
}
