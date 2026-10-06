import createDOMPurify, { type WindowLike } from "dompurify";
import { marked } from "marked";

let sanitize: ((html: string) => string) | null = null;

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function configureSanitizer(window: WindowLike) {
  const purifier = createDOMPurify(window);
  if (!purifier.isSupported) {
    // Fail closed: show the markdown as plain text rather than unsafe HTML.
    sanitize = null;
    return;
  }
  purifier.addHook("afterSanitizeAttributes", (node) => {
    if (node.tagName === "A") {
      node.setAttribute("target", "_blank");
      node.setAttribute("rel", "noopener noreferrer");
    }
  });
  sanitize = (html) =>
    purifier.sanitize(html, {
      FORBID_TAGS: ["img", "style", "iframe", "form", "input"],
    });
}

/**
 * Summary markdown to safe HTML. Summaries are model output, so they are
 * treated as untrusted input. Covered by the Playwright reader spec, which
 * runs in a real browser DOM.
 */
export function renderMarkdown(markdown: string): string {
  if (!markdown.trim()) return "";
  if (!sanitize && typeof window !== "undefined") {
    configureSanitizer(window as unknown as WindowLike);
  }
  if (!sanitize) {
    return `<p>${escapeHtml(markdown)}</p>`;
  }
  const html = marked.parse(markdown, { async: false, gfm: true });
  return sanitize(html);
}
