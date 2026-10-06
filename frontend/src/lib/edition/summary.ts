/**
 * Splits a generated summary into newspaper parts.
 *
 * Summaries follow the backend prompt: `## At a glance`, `## Overview`,
 * `## Key Points`, `## Takeaways`. The overview becomes the standfirst, the
 * glance list becomes a sidebar box, and everything else is the body.
 * Summaries that do not follow the format render whole as the body.
 */
export interface SummaryParts {
  /** Plain text under the headline. Empty when no overview exists. */
  standfirst: string;
  /** Markdown list for the "At a glance" box. Empty when absent. */
  glance: string;
  /** Remaining markdown, headings kept. */
  body: string;
}

interface MarkdownSection {
  heading: string | null;
  content: string;
}

const STANDFIRST_MAX_CHARS = 400;
const GLANCE_HEADING = /^at a glance$/i;
const OVERVIEW_HEADING = /^(brief )?overview$/i;

function splitSections(markdown: string): MarkdownSection[] {
  const sections: MarkdownSection[] = [];
  let current: MarkdownSection = { heading: null, content: "" };
  for (const line of markdown.replace(/\r\n/g, "\n").split("\n")) {
    const match = /^##\s+(.+?)\s*#*\s*$/.exec(line);
    if (match) {
      sections.push(current);
      current = { heading: match[1].trim(), content: "" };
    } else {
      current.content += `${line}\n`;
    }
  }
  sections.push(current);
  return sections.filter(
    (section) => section.heading !== null || section.content.trim(),
  );
}

/** Markdown inline syntax removed, whitespace collapsed. */
export function toPlainText(markdown: string): string {
  return markdown
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/[*_`]+/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function firstSentence(text: string): string {
  const match = /^.+?[.!?](?=\s|$)/.exec(text);
  return match ? match[0] : text;
}

function joinSections(sections: MarkdownSection[]): string {
  return sections
    .map((section) =>
      section.heading
        ? `## ${section.heading}\n${section.content}`
        : section.content,
    )
    .join("\n")
    .trim();
}

export function splitSummary(markdown: string): SummaryParts {
  const sections = splitSections(markdown);
  let glance = "";
  let standfirst = "";
  const body: MarkdownSection[] = [];

  for (const section of sections) {
    const heading = section.heading ?? "";
    if (!glance && GLANCE_HEADING.test(heading)) {
      glance = section.content.trim();
      continue;
    }
    if (!standfirst && OVERVIEW_HEADING.test(heading)) {
      const overview = toPlainText(section.content);
      if (overview.length <= STANDFIRST_MAX_CHARS) {
        standfirst = overview;
        continue;
      }
      // A long overview stays in the body; its opening sentence leads.
      standfirst = firstSentence(overview);
    }
    body.push(section);
  }

  return { standfirst, glance, body: joinSections(body) };
}
