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

const TEXT_NODE = 3;
const CONTENT_PREFIX_PATTERN = /^(?:Transcript|Summary):\s*/i;

function resolveNodeTextLength(node: Node) {
  return node.textContent?.length ?? 0;
}

function resolveBoundaryTextOffset(
  root: Node,
  container: Node,
  offset: number,
): number | null {
  let total = 0;
  let found = false;

  function visit(node: Node) {
    if (found) {
      return;
    }

    if (node === container) {
      if (node.nodeType === TEXT_NODE) {
        total += Math.min(offset, node.textContent?.length ?? 0);
        found = true;
        return;
      }

      const childNodes = Array.from(node.childNodes);
      const boundedOffset = Math.min(offset, childNodes.length);
      for (let index = 0; index < boundedOffset; index += 1) {
        total += resolveNodeTextLength(childNodes[index]);
      }
      found = true;
      return;
    }

    if (node.nodeType === TEXT_NODE) {
      total += node.textContent?.length ?? 0;
      return;
    }

    for (const child of Array.from(node.childNodes)) {
      visit(child);
      if (found) {
        return;
      }
    }
  }

  visit(root);
  return found ? total : null;
}

export function resolveRangeTextOffsets(root: Node, range: Range) {
  if (
    range.collapsed ||
    (range.startContainer !== root && !root.contains(range.startContainer)) ||
    (range.endContainer !== root && !root.contains(range.endContainer))
  ) {
    return null;
  }

  const start = resolveBoundaryTextOffset(
    root,
    range.startContainer,
    range.startOffset,
  );
  const end = resolveBoundaryTextOffset(
    root,
    range.endContainer,
    range.endOffset,
  );

  if (start === null || end === null || end <= start) {
    return null;
  }

  return { start, end };
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

function stripStoredContentPrefix(source: HighlightSource, value: string) {
  if ((source !== "transcript" && source !== "summary") || !value) {
    return value;
  }

  return value.replace(CONTENT_PREFIX_PATTERN, "").trimStart();
}

function normalizeLegacyHighlightFormatting(
  source: HighlightSource,
  value: string,
) {
  if ((source !== "transcript" && source !== "summary") || !value) {
    return value;
  }

  return value
    .replace(/\r\n?/g, "\n")
    .replace(/[ \t]{2,}\n/g, "")
    .replace(/\n{2,}/g, "\n");
}

function buildHighlightFieldVariants(source: HighlightSource, value: string) {
  const stripped = stripStoredContentPrefix(source, value);
  const variants = [
    value,
    stripped,
    normalizeLegacyHighlightFormatting(source, value),
    normalizeLegacyHighlightFormatting(source, stripped),
  ].filter((variant, index, all) => all.indexOf(variant) === index);

  return variants;
}

function resolveContextScore(
  actual: string,
  variants: string[],
  side: "prefix" | "suffix",
) {
  let bestScore = variants.includes("") ? 1 : 0;

  for (const variant of variants) {
    if (!variant) {
      continue;
    }

    const matches =
      side === "prefix" ? actual.endsWith(variant) : actual.startsWith(variant);

    if (matches) {
      bestScore = Math.max(bestScore, variant.length);
    }
  }

  return bestScore;
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
  const textVariants = buildHighlightFieldVariants(
    highlight.source,
    highlight.text,
  ).filter((variant) => Boolean(variant));
  const prefixVariants = buildHighlightFieldVariants(
    highlight.source,
    highlight.prefix_context,
  );
  const suffixVariants = buildHighlightFieldVariants(
    highlight.source,
    highlight.suffix_context,
  );

  for (const textVariant of textVariants) {
    let searchStart = 0;

    while (searchStart < fullText.length) {
      const start = fullText.indexOf(textVariant, searchStart);
      if (start < 0) {
        break;
      }

      const end = start + textVariant.length;
      const longestPrefix = Math.max(
        ...prefixVariants.map((variant) => variant.length),
        0,
      );
      const longestSuffix = Math.max(
        ...suffixVariants.map((variant) => variant.length),
        0,
      );
      const prefix = fullText.slice(Math.max(0, start - longestPrefix), start);
      const suffix = fullText.slice(
        end,
        Math.min(fullText.length, end + longestSuffix),
      );
      const score =
        resolveContextScore(prefix, prefixVariants, "prefix") +
        resolveContextScore(suffix, suffixVariants, "suffix");

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
