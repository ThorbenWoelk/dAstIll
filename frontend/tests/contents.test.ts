import { describe, expect, it } from "bun:test";

import { storyContents } from "../src/lib/edition/contents";

describe("storyContents", () => {
  it("lists the glance box and the body sections in order", () => {
    expect(
      storyContents(
        [
          { level: 2, title: "Key Points" },
          { level: 3, title: "Proposed Solutions" },
          { level: 2, title: "Takeaways" },
        ],
        true,
      ),
    ).toEqual([
      { title: "At a glance", heading: null },
      { title: "Key Points", heading: 0 },
      { title: "Takeaways", heading: 2 },
    ]);
  });

  it("uses the top level present in the body", () => {
    expect(
      storyContents(
        [
          { level: 3, title: "First" },
          { level: 3, title: "Second" },
        ],
        false,
      ),
    ).toEqual([
      { title: "First", heading: 0 },
      { title: "Second", heading: 1 },
    ]);
  });

  it("skips empty headings and collapses whitespace", () => {
    expect(
      storyContents(
        [
          { level: 2, title: "  " },
          { level: 2, title: "Key\n  Points" },
        ],
        true,
      ),
    ).toEqual([
      { title: "At a glance", heading: null },
      { title: "Key Points", heading: 1 },
    ]);
  });

  it("is empty when there is only one place to go", () => {
    expect(storyContents([{ level: 2, title: "Key Points" }], false)).toEqual(
      [],
    );
    expect(storyContents([], true)).toEqual([]);
    expect(storyContents([], false)).toEqual([]);
  });
});
