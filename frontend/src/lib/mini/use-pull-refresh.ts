export interface PullRefreshOptions {
  onRefresh: () => Promise<void> | void;
  threshold?: number;
  enabled?: boolean;
}

const DEFAULT_THRESHOLD_PX = 72;
const AXIS_LOCK_PX = 12;

export function pullRefresh(
  node: HTMLElement,
  options: PullRefreshOptions,
): { update(opts: PullRefreshOptions): void; destroy(): void } {
  let opts = options;
  let startX = 0;
  let startY = 0;
  let tracking = false;
  let refreshing = false;

  function reset() {
    startX = 0;
    startY = 0;
    tracking = false;
  }

  function isInteractive(el: EventTarget | null): boolean {
    if (!(el instanceof HTMLElement)) return false;
    return el.closest("button, a, input, select, textarea") !== null;
  }

  function onTouchStart(event: TouchEvent) {
    if (opts.enabled === false || refreshing) return;
    if (event.touches.length !== 1 || node.scrollTop > 0) return;
    if (isInteractive(event.target)) return;

    const touch = event.touches[0];
    startX = touch.clientX;
    startY = touch.clientY;
    tracking = true;
  }

  function onTouchMove(event: TouchEvent) {
    if (!tracking || event.touches.length !== 1) return;
    const touch = event.touches[0];
    const dx = touch.clientX - startX;
    const dy = touch.clientY - startY;

    if (dy < 0 || Math.abs(dx) > Math.abs(dy) || node.scrollTop > 0) {
      reset();
      return;
    }

    if (dy > AXIS_LOCK_PX && event.cancelable) {
      event.preventDefault();
    }
  }

  async function refresh() {
    refreshing = true;
    try {
      await opts.onRefresh();
    } finally {
      refreshing = false;
    }
  }

  function onTouchEnd(event: TouchEvent) {
    if (!tracking) return;
    const touch = event.changedTouches[0];
    const dx = touch.clientX - startX;
    const dy = touch.clientY - startY;
    const threshold = opts.threshold ?? DEFAULT_THRESHOLD_PX;
    reset();

    if (opts.enabled === false || refreshing) return;
    if (node.scrollTop > 0) return;
    if (dy < threshold || Math.abs(dx) > Math.abs(dy)) return;

    void refresh();
  }

  node.addEventListener("touchstart", onTouchStart, { passive: true });
  node.addEventListener("touchmove", onTouchMove, { passive: false });
  node.addEventListener("touchend", onTouchEnd, { passive: true });
  node.addEventListener("touchcancel", reset, { passive: true });

  return {
    update(newOpts: PullRefreshOptions) {
      opts = newOpts;
    },
    destroy() {
      node.removeEventListener("touchstart", onTouchStart);
      node.removeEventListener("touchmove", onTouchMove);
      node.removeEventListener("touchend", onTouchEnd);
      node.removeEventListener("touchcancel", reset);
    },
  };
}
