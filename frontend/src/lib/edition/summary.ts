/**
 * Splits a generated summary into newspaper parts.
 *
 * Summaries follow the backend prompt: `## At a glance` (older ones use
 * `## TL;DR`), `## Overview`, `## Key Points`, `## Takeaways`. The overview
 * becomes the standfirst, the glance list becomes a sidebar box, and
 * everything else is the body. Summaries that do not follow the format
 * render whole as the body. The rules are described in design.md under
 * "Summary layout".
 */
import { toPlainText } from "./markdown";

export interface SummaryParts {
  /** Plain text under the headline. Empty when no overview exists. */
  standfirst: string;
  /** Markdown list for the "At a glance" box. Empty when absent. */
  glance: string;
  /** Remaining markdown, headings kept. */
  body: string;
}

type SectionKind = "glance" | "overview" | "keyPoints" | "takeaways";

/** A heading (level 1 to 3) and the lines up to the next one. */
interface SummaryBlock {
  /** Heading level, or 0 for text before the first heading. */
  level: number;
  /** Heading text without markup, colon, or leading emoji. */
  title: string;
  /** The known section this heading names, if any. */
  kind: SectionKind | null;
  /** The heading line as written. Empty for text before the first heading. */
  headingLine: string;
  content: string;
}

const STANDFIRST_MAX_CHARS = 400;
/** Length of the standfirst cut from an overview without a sentence end. */
const STANDFIRST_TEASER_CHARS = 240;

const SECTION_NAMES: [SectionKind, RegExp][] = [
  [
    "glance",
    /^(at a glance|tl;? ?dr|summary at a glance|key takeaways at a glance|auf einen blick)$/,
  ],
  ["overview", /^(brief overview|overview|überblick|zusammenfassung)$/],
  ["keyPoints", /^(key points|kernpunkte|wichtigste punkte)$/],
  ["takeaways", /^(takeaways|key takeaways|fazit|erkenntnisse)$/],
];

/** Closing `#`s count only after a space, so `## Why C#` keeps its `#`. */
const HEADING_LINE = /^ {0,3}(#{1,6})(?:[ \t]+(.*?))?(?:[ \t]+#+)?[ \t]*$/;
const FENCE_OPEN = /^ {0,3}(`{3,}|~{3,})/;
const FENCE_CLOSE = /^ {0,3}(`{3,}|~{3,})[ \t]*$/;

/** A chat line before the summary, like "Here is the summary:". */
const CHAT_LEAD =
  /^\s*(?:here(?:'s| is| are)\b[^\n]*:|(?:sure|certainly|of course|okay)[,!.][^\n]*)\s*$/i;
const WHOLE_FENCE =
  /^\s*(`{3,}|~{3,})[ \t]*(?:markdown|md)?[ \t]*\n([\s\S]*)\n[ \t]*\1[ \t]*$/i;
const KNOWN_HEADING_ANYWHERE = /^ {0,3}#{1,3}[ \t]/m;

function headingTitle(text: string): string {
  return text
    .replace(/^[\s\p{P}\p{S}\p{Extended_Pictographic}\uFE0F\u200D]+/u, "")
    .replace(/^\d+[.)]\s+/, "")
    .replace(/[\s*_:：]+$/u, "")
    .replace(/\s+/g, " ")
    .trim();
}

function sectionKind(title: string): SectionKind | null {
  const name = title.toLowerCase();
  for (const [kind, pattern] of SECTION_NAMES) {
    if (pattern.test(name)) return kind;
  }
  return null;
}

/**
 * Removes model chatter around the summary: a leading `<think>` block, a
 * stray `</think>` tag, "Here is the summary:" lines, and a code fence
 * wrapped around the whole text.
 */
function removeModelChatter(markdown: string): string {
  let text = markdown
    .replace(/\r\n?/g, "\n")
    .replace(/^\s*<think>[\s\S]*?<\/think>/i, "")
    .replace(/(?<!`)<\/think>(?!`)/gi, "\n");

  const lines = text.split("\n");
  while (lines.length > 1 && (!lines[0].trim() || CHAT_LEAD.test(lines[0]))) {
    lines.shift();
  }
  text = lines.join("\n");

  const fenced = WHOLE_FENCE.exec(text.trimEnd());
  if (fenced && KNOWN_HEADING_ANYWHERE.test(fenced[2])) text = fenced[2];
  return text;
}

/** Cuts the text at headings of level 1 to 3, skipping code blocks. */
function splitBlocks(markdown: string): SummaryBlock[] {
  const blocks: SummaryBlock[] = [];
  let current: SummaryBlock = {
    level: 0,
    title: "",
    kind: null,
    headingLine: "",
    content: "",
  };
  let fence: string | null = null;

  for (const line of markdown.split("\n")) {
    if (fence) {
      const close = FENCE_CLOSE.exec(line);
      if (close && close[1][0] === fence[0] && close[1].length >= fence.length)
        fence = null;
      current.content += `${line}\n`;
      continue;
    }
    const open = FENCE_OPEN.exec(line);
    if (open) {
      fence = open[1];
      current.content += `${line}\n`;
      continue;
    }
    const heading = HEADING_LINE.exec(line);
    if (heading && heading[1].length <= 3) {
      blocks.push(current);
      const title = headingTitle(heading[2] ?? "");
      current = {
        level: heading[1].length,
        title,
        kind: sectionKind(title),
        headingLine: line,
        content: "",
      };
      continue;
    }
    current.content += `${line}\n`;
  }
  blocks.push(current);
  return blocks;
}

/**
 * A known name used as a subheading inside Key Points or Takeaways (for
 * example `### Overview` under `## Key Points`) is not a new section.
 */
function treatNestedSectionsAsSubheadings(blocks: SummaryBlock[]) {
  let open: SummaryBlock | null = null;
  for (const block of blocks) {
    if (!block.kind) continue;
    const insideList = open?.kind === "keyPoints" || open?.kind === "takeaways";
    if (open && insideList && block.level > open.level) {
      block.kind = null;
      continue;
    }
    open = block;
  }
}

/**
 * Index of the first block of the last complete section set. Some models
 * write a draft set and then the final one; a set restarts when a glance
 * or overview section follows a repeat of itself or the key points. A set
 * is complete when it has Key Points or Takeaways. Returns -1 when no
 * known section exists.
 */
function startOfFinalSummary(blocks: SummaryBlock[]): number {
  const sets: { start: number; complete: boolean }[] = [];
  let seen = new Set<SectionKind>();
  blocks.forEach((block, index) => {
    const kind = block.kind;
    if (!kind) return;
    const restarts =
      (kind === "glance" || kind === "overview") &&
      (seen.has(kind) || seen.has("keyPoints") || seen.has("takeaways"));
    if (!sets.length || restarts) {
      sets.push({ start: index, complete: false });
      seen = new Set();
    }
    seen.add(kind);
    if (kind === "keyPoints" || kind === "takeaways")
      sets.at(-1)!.complete = true;
  });
  if (!sets.length) return -1;
  return (sets.findLast((set) => set.complete) ?? sets[0]).start;
}

const ABBREVIATIONS = new Set([
  "dr",
  "mr",
  "mrs",
  "ms",
  "prof",
  "st",
  "vs",
  "etc",
  "jr",
  "sr",
  "mt",
  "ft",
  "no",
  "inc",
  "ltd",
  "co",
  "corp",
  "gen",
  "gov",
  "sen",
  "rep",
  "rev",
  "capt",
  "sgt",
  "lt",
  "col",
  "approx",
  "fig",
]);

/** Sentence end: `.`, `!` or `?`, closing quotes, space, then a new start. */
const SENTENCE_END = /[.!?]+["'”’)\]]*(?=\s+(\S))/gu;
const SENTENCE_START = /^[\p{Lu}\p{N}"'“‘([]/u;

function endsWithAbbreviation(text: string): boolean {
  const word = /[\p{L}.]*$/u.exec(text)?.[0] ?? "";
  return (
    ABBREVIATIONS.has(word.toLowerCase()) ||
    /^\p{L}$/u.test(word) ||
    /^(\p{L}\.)+\p{L}$/u.test(word)
  );
}

/** The first sentence, or null when the text has no clear sentence end. */
export function firstSentence(text: string): string | null {
  for (const match of text.matchAll(SENTENCE_END)) {
    if (!SENTENCE_START.test(match[1])) continue;
    const before = text.slice(0, match.index);
    if (match[0][0] === "." && endsWithAbbreviation(before)) continue;
    return text.slice(0, match.index + match[0].length);
  }
  return /[.!?]["'”’)\]]*$/u.test(text) ? text : null;
}

function cutAtWord(text: string, maxChars: number): string {
  if (text.length <= maxChars) return text;
  const cut = text.slice(0, maxChars + 1);
  const end = cut.lastIndexOf(" ");
  const words = end > 0 ? cut.slice(0, end) : text.slice(0, maxChars);
  return `${words.replace(/[\s,;:–—-]+$/u, "")}…`;
}

/** Opening of an overview too long to be the whole standfirst. */
function overviewLead(overview: string): string {
  const sentence = firstSentence(overview);
  if (sentence && sentence.length <= STANDFIRST_MAX_CHARS) return sentence;
  return cutAtWord(overview, STANDFIRST_TEASER_CHARS);
}

function blockMarkdown(block: SummaryBlock): string {
  if (!block.headingLine) return block.content;
  const heading = block.kind ? `## ${block.title}` : block.headingLine;
  return `${heading}\n${block.content}`;
}

export function splitSummary(markdown: string): SummaryParts {
  const blocks = splitBlocks(removeModelChatter(markdown));
  treatNestedSectionsAsSubheadings(blocks);
  const start = startOfFinalSummary(blocks);
  const kept = start === -1 ? blocks : blocks.slice(start);

  let glance: string | null = null;
  let standfirst: string | null = null;
  const body: string[] = [];

  for (const block of kept) {
    if (block.kind === "glance" && glance === null) {
      glance = block.content.trim();
      continue;
    }
    if (block.kind === "overview" && standfirst === null) {
      const overview = toPlainText(block.content);
      if (overview.length <= STANDFIRST_MAX_CHARS) {
        standfirst = overview;
        continue;
      }
      // A long overview stays in the body; its opening leads.
      standfirst = overviewLead(overview);
    }
    body.push(blockMarkdown(block));
  }

  return {
    standfirst: standfirst ?? "",
    glance: glance ?? "",
    body: body.join("").trim(),
  };
}
