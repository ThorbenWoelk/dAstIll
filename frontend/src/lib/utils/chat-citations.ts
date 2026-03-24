import { buildWorkspaceViewHref } from "$lib/view-url";
import type { ChatSource } from "$lib/types";

const MARKER_RE = /\[(?:Source\s*)?(\d{1,2})\]/gi;
const MAX_CITE_QUERY_LEN = 96;

function citeSnippetForUrl(snippet: string): string {
  const t = snippet.trim().replace(/\s+/g, " ");
  if (!t) return "";
  return t.length <= MAX_CITE_QUERY_LEN ? t : t.slice(0, MAX_CITE_QUERY_LEN);
}

/** Workspace URL that opens the transcript/summary tab and scrolls to the excerpt text when possible. */
export function buildChatSourceWorkspaceHref(source: ChatSource): string {
  const cite = citeSnippetForUrl(source.snippet);
  return buildWorkspaceViewHref({
    selectedChannelId: source.channel_id,
    selectedVideoId: source.video_id,
    contentMode: source.source_kind,
    videoTypeFilter: "all",
    acknowledgedFilter: "all",
    chunkId: source.chunk_id || undefined,
    citeQuery: cite || undefined,
  });
}

function escapeAttr(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
}

/** True when the assistant text uses bracket citation markers like [1] or [Source 2]. */
export function hasCitationMarkers(raw: string): boolean {
  return /\[(?:Source\s*)?\d{1,2}\]/.test(raw);
}

/**
 * Turn [n] / [Source n] in rendered HTML into links to workspace sources when indices match.
 */
export function linkifyCitationMarkers(
  html: string,
  sources: ChatSource[],
): string {
  if (!sources.length) return html;
  return html.replace(MARKER_RE, (match, n) => {
    const index = parseInt(n, 10) - 1;
    if (index < 0 || index >= sources.length) return match;
    const s = sources[index];
    const href = buildChatSourceWorkspaceHref(s);
    const tip = [s.video_title, s.section_title].filter(Boolean).join(" · ");
    return `<sup class="chat-cite-sup"><a href="${escapeAttr(href)}" class="chat-cite-ref" data-tooltip="${escapeAttr(tip)}" target="_blank" rel="noopener noreferrer">${match}</a></sup>`;
  });
}

export function sourceWorkspaceHref(source: ChatSource): string {
  return buildChatSourceWorkspaceHref(source);
}

export function sourceTooltipTitle(source: ChatSource): string {
  return [source.video_title, source.section_title].filter(Boolean).join(" · ");
}
