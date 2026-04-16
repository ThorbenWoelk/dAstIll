export interface MiniKeyboardContext {
  stepSummary: (delta: -1 | 1) => void;
  markActiveSummaryRead: () => Promise<void>;
  activeSummary: { read: boolean } | null;
  channelSheetOpen: boolean;
  closeChannelSheet: () => void;
}

export function createMiniKeydownHandler(
  getContext: () => MiniKeyboardContext,
): (event: KeyboardEvent) => void {
  return (event: KeyboardEvent) => {
    if (
      event.target instanceof HTMLSelectElement ||
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return;
    }

    const ctx = getContext();

    switch (event.key) {
      case "Escape":
        if (ctx.channelSheetOpen) {
          event.preventDefault();
          ctx.closeChannelSheet();
        }
        break;
      case "ArrowLeft":
      case "j":
        event.preventDefault();
        ctx.stepSummary(-1);
        break;
      case "ArrowRight":
      case "k":
        event.preventDefault();
        ctx.stepSummary(1);
        break;
      case "r":
        if (ctx.activeSummary && !ctx.activeSummary.read) {
          event.preventDefault();
          void ctx.markActiveSummaryRead();
        }
        break;
    }
  };
}
