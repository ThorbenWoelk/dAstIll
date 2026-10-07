import type { CreateHighlightRequest, Highlight } from "$lib/api";

/**
 * Highlights are stored as text plus a little context on either side, never
 * as offsets. That keeps them valid when the summary is re-rendered, and
 * lets a highlight find its place again even if the markup changes.
 */

/** Characters of context saved before and after the highlighted text. */
const CONTEXT_CHARS = 80;

export interface TextSpan {
  start: number;
  end: number;
}

export interface PlacedHighlight extends TextSpan {
  id: string;
}

/** Turns a selected span of the story text into a highlight to save. */
export function buildHighlightDraft(
  text: string,
  start: number,
  end: number,
): CreateHighlightRequest | null {
  let from = Math.max(0, start);
  let to = Math.min(text.length, end);
  while (from < to && /\s/.test(text[from])) from++;
  while (to > from && /\s/.test(text[to - 1])) to--;
  if (to <= from) return null;
  return {
    source: "summary",
    text: text.slice(from, to),
    prefix_context: text.slice(Math.max(0, from - CONTEXT_CHARS), from),
    suffix_context: text.slice(to, to + CONTEXT_CHARS),
  };
}

interface FoldedText {
  text: string;
  /** `origins[i]` is where folded character `i` sits in the original. */
  origins: number[];
}

/** Collapses every run of whitespace to one space, remembering positions. */
function foldWhitespace(text: string): FoldedText {
  let folded = "";
  const origins: number[] = [];
  let inSpace = false;
  for (let index = 0; index < text.length; index++) {
    const char = text[index];
    if (/\s/.test(char)) {
      if (inSpace) continue;
      inSpace = true;
      folded += " ";
    } else {
      inSpace = false;
      folded += char;
    }
    origins.push(index);
  }
  return { text: folded, origins };
}

/** How many characters `before` shares with the end of `context`. */
function sharedEnding(before: string, context: string): number {
  let count = 0;
  while (
    count < before.length &&
    count < context.length &&
    before[before.length - 1 - count] === context[context.length - 1 - count]
  ) {
    count++;
  }
  return count;
}

/** How many characters `after` shares with the start of `context`. */
function sharedStart(after: string, context: string): number {
  let count = 0;
  while (
    count < after.length &&
    count < context.length &&
    after[count] === context[count]
  ) {
    count++;
  }
  return count;
}

function placeOne(passage: FoldedText, highlight: Highlight): TextSpan | null {
  const needle = foldWhitespace(highlight.text.trim()).text;
  if (!needle) return null;
  const prefix = foldWhitespace(highlight.prefix_context).text;
  const suffix = foldWhitespace(highlight.suffix_context).text;

  let best: (TextSpan & { score: number }) | null = null;
  for (
    let at = passage.text.indexOf(needle);
    at >= 0;
    at = passage.text.indexOf(needle, at + 1)
  ) {
    const end = at + needle.length;
    const score =
      sharedEnding(
        passage.text.slice(Math.max(0, at - prefix.length), at),
        prefix,
      ) + sharedStart(passage.text.slice(end, end + suffix.length), suffix);
    if (!best || score > best.score) best = { start: at, end, score };
  }
  if (!best) return null;
  return {
    start: passage.origins[best.start],
    end: passage.origins[best.end - 1] + 1,
  };
}

/**
 * Finds where each highlight sits in the story text. Highlights that no
 * longer match, or that overlap one placed earlier, are left out.
 */
export function placeHighlights(
  text: string,
  highlights: Highlight[],
): PlacedHighlight[] {
  const passage = foldWhitespace(text);
  const placed: PlacedHighlight[] = [];
  for (const highlight of highlights) {
    const span = placeOne(passage, highlight);
    if (!span) continue;
    if (
      placed.some((other) => span.start < other.end && span.end > other.start)
    ) {
      continue;
    }
    placed.push({ id: highlight.id, ...span });
  }
  return placed.sort((left, right) => left.start - right.start);
}
