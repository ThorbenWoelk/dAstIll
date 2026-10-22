/**
 * The contents line under a story's byline: "At a glance" and the body's
 * sections, so a long story can be skipped through. The rules are
 * described in design.md under "Story contents".
 */

/** A heading as rendered in the story body. */
export interface BodyHeading {
  /** 1 to 3, from the `h1` to `h3` tag. */
  level: number;
  title: string;
}

export interface ContentsEntry {
  title: string;
  /** Index into the body headings, or null for the "At a glance" box. */
  heading: number | null;
}

export const GLANCE_TITLE = "At a glance";

/** Fewer entries than this are not worth a contents line. */
const MIN_ENTRIES = 2;

/**
 * The glance box first, then the body headings of the top level used in
 * the body (usually the `##` sections like Key Points and Takeaways).
 * Subheadings below that level are left out. Empty when the story has
 * fewer than two places to go.
 */
export function storyContents(
  headings: BodyHeading[],
  hasGlance: boolean,
): ContentsEntry[] {
  const titled = headings
    .map((heading, index) => ({ ...heading, index }))
    .filter((heading) => heading.title.trim());
  const topLevel = Math.min(...titled.map((heading) => heading.level));
  const entries: ContentsEntry[] = [
    ...(hasGlance ? [{ title: GLANCE_TITLE, heading: null }] : []),
    ...titled
      .filter((heading) => heading.level === topLevel)
      .map((heading) => ({
        title: heading.title.replace(/\s+/g, " ").trim(),
        heading: heading.index,
      })),
  ];
  return entries.length >= MIN_ENTRIES ? entries : [];
}
