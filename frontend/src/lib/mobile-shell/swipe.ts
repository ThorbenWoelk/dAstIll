const SWIPE_BACK_THRESHOLD_PX = 72;
const SWIPE_BACK_EDGE_PX = 32;

interface TouchGesture {
  startX: number;
  startY: number;
  edgeStart: boolean;
  interactive: boolean;
}

function isInteractiveSwipeTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(
    target.closest(
      "button, a, input, textarea, select, label, [role='button'], [role='tab']",
    ),
  );
}

export interface SwipeBackOptions {
  enabled: boolean;
  onBack: () => void;
}

/**
 * Svelte action that triggers `onBack` when the user swipes right from the
 * left edge. Mirrors the swipe-back gesture used in iOS-style mobile shells.
 */
export function swipeBack(node: HTMLElement, opts: SwipeBackOptions) {
  let options = opts;
  let touchGesture: TouchGesture | null = null;

  function handleStart(event: TouchEvent) {
    if (!options.enabled || event.touches.length !== 1) {
      touchGesture = null;
      return;
    }
    const touch = event.touches[0];
    const edgeStart = touch.clientX <= SWIPE_BACK_EDGE_PX;
    touchGesture = {
      startX: touch.clientX,
      startY: touch.clientY,
      edgeStart,
      interactive: edgeStart ? false : isInteractiveSwipeTarget(event.target),
    };
  }

  function handleEnd(event: TouchEvent) {
    if (
      !touchGesture ||
      !touchGesture.edgeStart ||
      touchGesture.interactive ||
      !options.enabled ||
      event.changedTouches.length !== 1
    ) {
      touchGesture = null;
      return;
    }
    const touch = event.changedTouches[0];
    const deltaX = touch.clientX - touchGesture.startX;
    const deltaY = touch.clientY - touchGesture.startY;
    touchGesture = null;
    if (
      deltaX < SWIPE_BACK_THRESHOLD_PX ||
      Math.abs(deltaX) <= Math.abs(deltaY) * 1.25
    ) {
      return;
    }
    options.onBack();
  }

  function handleCancel() {
    touchGesture = null;
  }

  node.addEventListener("touchstart", handleStart, { passive: true });
  node.addEventListener("touchend", handleEnd);
  node.addEventListener("touchcancel", handleCancel);

  return {
    update(newOptions: SwipeBackOptions) {
      options = newOptions;
    },
    destroy() {
      node.removeEventListener("touchstart", handleStart);
      node.removeEventListener("touchend", handleEnd);
      node.removeEventListener("touchcancel", handleCancel);
    },
  };
}
