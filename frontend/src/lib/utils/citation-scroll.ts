/**
 * Scroll the workspace transcript/summary article so the first occurrence of
 * a citation snippet (from chat source excerpts) is brought into view.
 */
export function scrollToCitationInArticle(
  root: HTMLElement,
  query: string,
): boolean {
  const normalized = normalizeForMatch(query);
  if (!normalized) return false;

  const candidates: string[] = [normalized];
  if (normalized.length > 40) {
    candidates.push(normalized.slice(0, 80));
    candidates.push(normalized.slice(0, 48));
  }

  for (const needle of candidates) {
    if (!needle) continue;
    if (tryScrollToNeedle(root, needle)) return true;
  }
  return false;
}

function normalizeForMatch(s: string): string {
  return s.replace(/\s+/g, " ").trim().toLowerCase();
}

function tryScrollToNeedle(root: HTMLElement, needle: string): boolean {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let node: Text | null;
  while ((node = walker.nextNode() as Text | null)) {
    const data = node.data;
    if (!data) continue;
    const lower = data.toLowerCase();
    const idx = lower.indexOf(needle);
    if (idx === -1) continue;
    const end = Math.min(idx + needle.length, data.length);
    const range = document.createRange();
    range.setStart(node, idx);
    range.setEnd(node, end);
    const el = range.startContainer.parentElement;
    if (el) {
      el.scrollIntoView({ block: "center", behavior: "smooth" });
    }
    return true;
  }
  return false;
}
