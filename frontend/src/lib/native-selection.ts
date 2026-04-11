declare global {
  interface Window {
    __tauri_selection_highlight?: () => void;
    __tauri_selection_correct?: () => void;
  }
}

export function registerNativeSelectionHandlers(
  onHighlight: () => void,
  onCorrect: () => void,
): () => void {
  if (typeof window === "undefined") {
    return () => undefined;
  }

  const previousHighlight = window.__tauri_selection_highlight;
  const previousCorrect = window.__tauri_selection_correct;

  window.__tauri_selection_highlight = onHighlight;
  window.__tauri_selection_correct = onCorrect;

  return () => {
    window.__tauri_selection_highlight = previousHighlight;
    window.__tauri_selection_correct = previousCorrect;
  };
}
