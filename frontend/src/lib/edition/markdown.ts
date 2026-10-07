import createDOMPurify, { type WindowLike } from "dompurify";
import { Marked, type Token } from "marked";

/**
 * A timestamp in brackets, like `[12:34]` or `[1:02:03 - 1:05:00]`.
 * Summaries quote video chapters this way; it is never a link label.
 */
const TIMESTAMP_DEFINITION =
  /^ {0,3}\[\s*\d{1,2}:\d{2}(?::\d{2})?(?:\s*[-–—]\s*\d{1,2}:\d{2}(?::\d{2})?)?\s*\]:/;

/**
 * Markdown settings for summaries:
 * - Raw HTML is not recognised, so `<PricingTable />` or `<script src=…>`
 *   written as prose stays visible as text instead of being removed.
 * - `[12:34]: Intro` is not read as a link reference definition. Without
 *   that definition, `[12:34] text` stays plain text too.
 */
const summaryMarkdown = new Marked({
  gfm: true,
  tokenizer: {
    html: () => undefined,
    tag: () => undefined,
    def(src) {
      if (TIMESTAMP_DEFINITION.test(src)) return undefined;
      return false;
    },
  },
});

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
 * treated as untrusted input. Raw HTML in the markdown shows as text, and
 * DOMPurify still cleans the result. Covered by the Playwright reader spec,
 * which runs in a real browser DOM.
 */
export function renderMarkdown(markdown: string): string {
  if (!markdown.trim()) return "";
  if (!sanitize && typeof window !== "undefined") {
    configureSanitizer(window as unknown as WindowLike);
  }
  if (!sanitize) {
    return `<p>${escapeHtml(markdown)}</p>`;
  }
  const html = summaryMarkdown.parse(markdown, { async: false });
  return sanitize(html);
}

const NAMED_ENTITIES: Record<string, string> = {
  amp: "&",
  lt: "<",
  gt: ">",
  quot: '"',
  apos: "'",
  nbsp: " ",
};

function decodeEntities(text: string): string {
  return text.replace(
    /&(#\d+|#x[0-9a-f]+|[a-z]+);/gi,
    (entity, name: string) => {
      if (name[0] !== "#") return NAMED_ENTITIES[name.toLowerCase()] ?? entity;
      const code =
        name[1] === "x" || name[1] === "X"
          ? Number.parseInt(name.slice(2), 16)
          : Number.parseInt(name.slice(1), 10);
      return code > 0 && code <= 0x10ffff ? String.fromCodePoint(code) : entity;
    },
  );
}

/** Tokens that hold other blocks; their children are separate passages. */
const BLOCK_CONTAINERS = new Set(["blockquote", "list_item"]);

function tokenText(token: Token): string {
  switch (token.type) {
    case "space":
    case "hr":
    case "br":
    case "def":
      return " ";
    case "checkbox":
      return "";
    case "list":
      return token.items.map(tokenText).join(" ");
    case "table":
      return [token.header, ...token.rows]
        .flat()
        .map((cell) => cell.tokens.map(tokenText).join(""))
        .join(" ");
  }
  if ("tokens" in token && token.tokens?.length) {
    const separator = BLOCK_CONTAINERS.has(token.type) ? " " : "";
    return token.tokens.map(tokenText).join(separator);
  }
  return "text" in token && typeof token.text === "string" ? token.text : "";
}

/**
 * Markdown as the words a reader sees: syntax removed, escapes and
 * entities resolved, whitespace collapsed. Uses the same parser as the
 * rendered article, so text like `5 * 3` or `snake_case` survives.
 */
export function toPlainText(markdown: string): string {
  const tokens = summaryMarkdown.lexer(markdown);
  return decodeEntities(tokens.map(tokenText).join(" "))
    .replace(/\s+/g, " ")
    .trim();
}
