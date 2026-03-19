import type { Highlight } from "$lib/types";

export function mergeVideoHighlights(
  highlightsByVideoId: Record<string, Highlight[]>,
  videoId: string,
  highlight: Highlight,
) {
  return {
    ...highlightsByVideoId,
    [videoId]: [...(highlightsByVideoId[videoId] ?? []), highlight],
  };
}

export function removeVideoHighlightFromState(
  highlightsByVideoId: Record<string, Highlight[]>,
  videoId: string,
  highlightId: number,
) {
  return {
    ...highlightsByVideoId,
    [videoId]: (highlightsByVideoId[videoId] ?? []).filter(
      (highlight) => highlight.id !== highlightId,
    ),
  };
}
