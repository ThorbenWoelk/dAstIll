import type {
  CreateHighlightRequest,
  Highlight,
  HighlightChannelGroup,
  HighlightSource,
} from "../types";

export interface HighlightRange {
  highlightId: number;
  start: number;
  end: number;
}

interface TooltipRect {
  left: number;
  top: number;
  width: number;
}

export function buildHighlightDraft(
  fullText: string,
  source: HighlightSource,
  start: number,
  end: number,
  contextWindow = 80,
): CreateHighlightRequest | null {
  if (start < 0 || end <= start || end > fullText.length) {
    return null;
  }

  const text = fullText.slice(start, end).trim();
  if (!text) {
    return null;
  }

  return {
    source,
    text,
    prefix_context: fullText.slice(Math.max(0, start - contextWindow), start),
    suffix_context: fullText.slice(
      end,
      Math.min(fullText.length, end + contextWindow),
    ),
  };
}

function resolveBestMatch(fullText: string, highlight: Highlight) {
  if (!highlight.text) {
    return null;
  }

  let best: {
    start: number;
    end: number;
    score: number;
  } | null = null;
  let searchStart = 0;

  while (searchStart < fullText.length) {
    const start = fullText.indexOf(highlight.text, searchStart);
    if (start < 0) {
      break;
    }

    const end = start + highlight.text.length;
    let score = 0;

    if (highlight.prefix_context) {
      const prefix = fullText.slice(
        Math.max(0, start - highlight.prefix_context.length),
        start,
      );
      if (prefix.endsWith(highlight.prefix_context)) {
        score += highlight.prefix_context.length;
      }
    } else {
      score += 1;
    }

    if (highlight.suffix_context) {
      const suffix = fullText.slice(
        end,
        Math.min(fullText.length, end + highlight.suffix_context.length),
      );
      if (suffix.startsWith(highlight.suffix_context)) {
        score += highlight.suffix_context.length;
      }
    } else {
      score += 1;
    }

    const candidate = { start, end, score };
    if (
      !best ||
      candidate.score > best.score ||
      (candidate.score === best.score && candidate.start < best.start)
    ) {
      best = candidate;
    }

    searchStart = start + 1;
  }

  return best;
}

export function resolveHighlightRanges(
  fullText: string,
  highlights: Highlight[],
): HighlightRange[] {
  const accepted: HighlightRange[] = [];

  for (const highlight of highlights) {
    const match = resolveBestMatch(fullText, highlight);
    if (!match) {
      continue;
    }

    const overlapsExisting = accepted.some(
      (range) => match.start < range.end && match.end > range.start,
    );
    if (overlapsExisting) {
      continue;
    }

    accepted.push({
      highlightId: highlight.id,
      start: match.start,
      end: match.end,
    });
  }

  return accepted.sort((a, b) => a.start - b.start);
}

function sortHighlightsNewestFirst(highlights: Highlight[]) {
  return highlights.toSorted((a, b) => {
    const byCreatedAt = b.created_at.localeCompare(a.created_at);
    return byCreatedAt !== 0 ? byCreatedAt : b.id - a.id;
  });
}

export function buildOptimisticHighlight(
  videoId: string,
  payload: CreateHighlightRequest,
  optimisticId: number,
  createdAt = new Date().toISOString(),
): Highlight {
  return {
    id: optimisticId,
    video_id: videoId,
    source: payload.source,
    text: payload.text,
    prefix_context: payload.prefix_context,
    suffix_context: payload.suffix_context,
    created_at: createdAt,
  };
}

export function mergeHighlightIntoList(
  current: Highlight[],
  highlight: Highlight,
) {
  return sortHighlightsNewestFirst([
    highlight,
    ...current.filter((item) => item.id !== highlight.id),
  ]);
}

export function reconcileOptimisticHighlight(
  current: Highlight[],
  optimisticId: number,
  confirmedHighlight: Highlight,
) {
  return mergeHighlightIntoList(
    current.filter(
      (item) => item.id !== optimisticId && item.id !== confirmedHighlight.id,
    ),
    confirmedHighlight,
  );
}

export function removeHighlightFromGroups(
  groups: HighlightChannelGroup[],
  highlightId: number,
) {
  return groups
    .map((group) => ({
      ...group,
      videos: group.videos
        .map((video) => ({
          ...video,
          highlights: video.highlights.filter(
            (highlight) => Number(highlight.id) !== highlightId,
          ),
        }))
        .filter((video) => video.highlights.length > 0),
    }))
    .filter((group) => group.videos.length > 0);
}

export function resolveTooltipPosition(
  anchorRect: TooltipRect,
  containerRect: TooltipRect,
  options: {
    horizontalInset?: number;
    topOffset?: number;
    topInset?: number;
  } = {},
) {
  const horizontalInset = options.horizontalInset ?? 80;
  const topOffset = options.topOffset ?? 46;
  const topInset = options.topInset ?? 16;
  const relativeCenter =
    anchorRect.left + anchorRect.width / 2 - containerRect.left;
  const minLeft = Math.min(horizontalInset, containerRect.width / 2);
  const maxLeft = Math.max(containerRect.width - horizontalInset, minLeft);

  return {
    left: Math.min(Math.max(relativeCenter, minLeft), maxLeft),
    top: Math.max(anchorRect.top - containerRect.top - topOffset, topInset),
  };
}
