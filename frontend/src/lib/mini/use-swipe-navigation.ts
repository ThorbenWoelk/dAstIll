export interface SwipeNavigationOptions {
  onSwipeLeft: () => void;
  onSwipeRight: () => void;
  threshold?: number;
  edgeIgnore?: number;
  enabled?: boolean;
}

export function swipeNavigation(
  node: HTMLElement,
  options: SwipeNavigationOptions,
): { update(opts: SwipeNavigationOptions): void; destroy(): void } {
  let opts = options;
  let startX = 0;
  let startY = 0;

  function isInteractive(el: EventTarget | null): boolean {
    if (!(el instanceof HTMLElement)) return false;
    const tag = el.tagName;
    return (
      tag === "BUTTON" ||
      tag === "A" ||
      tag === "INPUT" ||
      tag === "SELECT" ||
      tag === "TEXTAREA" ||
      el.closest("button, a, input, select, textarea") !== null
    );
  }

  function onTouchStart(e: TouchEvent) {
    if (opts.enabled === false) return;
    if (isInteractive(e.target)) return;
    const touch = e.touches[0];
    startX = touch.clientX;
    startY = touch.clientY;
  }

  function onTouchEnd(e: TouchEvent) {
    if (opts.enabled === false) return;
    const touch = e.changedTouches[0];
    const dx = touch.clientX - startX;
    const dy = touch.clientY - startY;
    const edgeIgnore = opts.edgeIgnore ?? 40;
    const threshold = opts.threshold ?? 60;

    if (startX <= edgeIgnore) return;
    if (Math.abs(dx) < threshold) return;
    if (Math.abs(dy) > Math.abs(dx) * 0.8) return;

    if (dx < 0) {
      opts.onSwipeLeft();
    } else {
      opts.onSwipeRight();
    }
  }

  node.addEventListener("touchstart", onTouchStart, { passive: true });
  node.addEventListener("touchend", onTouchEnd, { passive: true });

  return {
    update(newOpts: SwipeNavigationOptions) {
      opts = newOpts;
    },
    destroy() {
      node.removeEventListener("touchstart", onTouchStart);
      node.removeEventListener("touchend", onTouchEnd);
    },
  };
}
