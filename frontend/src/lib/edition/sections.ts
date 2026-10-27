/**
 * The story body cut into sections, each under one of its top-level
 * headings, so the reader can fold a section away to skip it. The rules
 * are described in design.md under "Story sections".
 */
import { toPlainText } from "./markdown";
import { splitBlocks, splitSummary } from "./summary";

export interface StorySection {
  /** Heading text as the reader sees it, or null for text before it. */
  title: string | null;
  /** 1 to 3, from the `#` count. 0 for text before the first heading. */
  level: number;
  /** Markdown under the heading, subheadings included. */
  markdown: string;
}

/**
 * Sections start at the top heading level used in the body (usually the
 * `##` sections like Key Points and Takeaways). Lower headings stay inside
 * their section. Text before the first heading is a section without title.
 */
export function storySections(body: string): StorySection[] {
  const blocks = splitBlocks(body).map((block) => ({
    ...block,
    text: block.headingLine ? toPlainText(block.headingLine) : "",
  }));
  const titled = blocks.filter((block) => block.text);
  const topLevel = Math.min(...titled.map((block) => block.level));

  const sections: StorySection[] = [];
  let current: StorySection = { title: null, level: 0, markdown: "" };
  for (const block of blocks) {
    if (block.text && block.level === topLevel) {
      sections.push(current);
      current = {
        title: block.text,
        level: block.level,
        markdown: block.content,
      };
      continue;
    }
    const heading = block.headingLine ? `${block.headingLine}\n` : "";
    current.markdown += `${heading}${block.content}`;
  }
  sections.push(current);

  return sections
    .map((section) => ({ ...section, markdown: section.markdown.trim() }))
    .filter((section) => section.title !== null || section.markdown);
}

/** A heading in the story the reader can jump to from the margin. */
export interface StoryPart {
  /** The element id of the part's section. */
  id: string;
  title: string;
}

export const GLANCE_PART_ID = "story-part-glance";

/** The element id of a body section, by its index in `storySections`. */
export function storyPartId(index: number): string {
  return `story-part-${index}`;
}

/** "At a glance" when the story has one, then every titled body section. */
export function outlineStory(summary: string): StoryPart[] {
  const parts = splitSummary(summary);
  const outline: StoryPart[] = parts.glance.trim()
    ? [{ id: GLANCE_PART_ID, title: "At a glance" }]
    : [];
  storySections(parts.body).forEach((section, index) => {
    if (section.title) {
      outline.push({ id: storyPartId(index), title: section.title });
    }
  });
  return outline;
}
