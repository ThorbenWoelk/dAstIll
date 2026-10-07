import type { PlacedHighlight, TextSpan } from "$lib/highlights/anchoring";

/**
 * The readable text of a story as one string, with a line break between
 * blocks so a highlight across two paragraphs reads naturally. Elements
 * marked `data-passage-skip` (headline, byline, buttons) are left out.
 */
export interface Passage {
  text: string;
  segments: { node: Text; start: number }[];
}

const BLOCK_TAGS = new Set([
  "ARTICLE",
  "ASIDE",
  "BLOCKQUOTE",
  "DIV",
  "H1",
  "H2",
  "H3",
  "H4",
  "LI",
  "OL",
  "P",
  "PRE",
  "SECTION",
  "UL",
]);

const MARK_CLASS = "reader-highlight";

function blockOf(node: Node, root: Element): Element {
  let element = node.parentElement;
  while (element && element !== root && !BLOCK_TAGS.has(element.tagName)) {
    element = element.parentElement;
  }
  return element ?? root;
}

export function readPassage(root: Element): Passage {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const segments: Passage["segments"] = [];
  let text = "";
  let lastBlock: Element | null = null;
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const textNode = node as Text;
    if (!textNode.data) continue;
    if (textNode.parentElement?.closest("[data-passage-skip]")) continue;
    const block = blockOf(textNode, root);
    if (lastBlock && block !== lastBlock && !text.endsWith("\n")) {
      text += "\n";
    }
    segments.push({ node: textNode, start: text.length });
    text += textNode.data;
    lastBlock = block;
  }
  return { text, segments };
}

function offsetAt(passage: Passage, container: Node, offset: number): number {
  const segment = passage.segments.find((s) => s.node === container);
  if (segment) return segment.start + Math.min(offset, segment.node.length);
  // The boundary sits between nodes: use the next text that follows it.
  const point = document.createRange();
  point.setStart(container, offset);
  for (const candidate of passage.segments) {
    if (point.comparePoint(candidate.node, 0) >= 0) return candidate.start;
  }
  return passage.text.length;
}

/** Where a selection falls in the passage, or null when it is outside. */
export function spanOfRange(
  passage: Passage,
  root: Element,
  range: Range,
): TextSpan | null {
  if (
    !root.contains(range.startContainer) ||
    !root.contains(range.endContainer)
  ) {
    return null;
  }
  const start = offsetAt(passage, range.startContainer, range.startOffset);
  const end = offsetAt(passage, range.endContainer, range.endOffset);
  return end > start ? { start, end } : null;
}

/** Wraps each placed highlight in `<mark>`, one per text node it covers. */
export function markHighlights(passage: Passage, placed: PlacedHighlight[]) {
  // Work backwards so splitting a text node never moves an earlier span.
  const latestFirst = [...placed].sort((a, b) => b.start - a.start);
  for (const span of latestFirst) {
    const touched = passage.segments.filter(
      (s) => s.start < span.end && s.start + s.node.length > span.start,
    );
    for (const segment of touched.reverse()) {
      const from = Math.max(span.start - segment.start, 0);
      const to = Math.min(span.end - segment.start, segment.node.length);
      if (to <= from || !segment.node.data.slice(from, to).trim()) continue;
      let target = segment.node;
      if (from > 0) target = target.splitText(from);
      if (to - from < target.length) target.splitText(to - from);
      const mark = document.createElement("mark");
      mark.className = MARK_CLASS;
      mark.dataset.highlightId = span.id;
      target.before(mark);
      mark.append(target);
    }
  }
}

export function clearHighlightMarks(root: Element) {
  for (const mark of root.querySelectorAll(`mark.${MARK_CLASS}`)) {
    mark.replaceWith(...mark.childNodes);
  }
  root.normalize();
}

/** The highlight id of a mark under an event target, if any. */
export function highlightIdAt(target: EventTarget | null): string | null {
  if (!(target instanceof Element)) return null;
  const mark = target.closest<HTMLElement>(`mark.${MARK_CLASS}`);
  return mark?.dataset.highlightId ?? null;
}
