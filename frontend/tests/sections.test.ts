import { describe, expect, it } from "bun:test";

import { storySections } from "../src/lib/edition/sections";

describe("storySections", () => {
  it("cuts the body at its top-level headings", () => {
    expect(
      storySections(
        "## Key Points\nFirst.\n\n### Proposed Solutions\nMore.\n\n## Takeaways\n- Remember.\n",
      ),
    ).toEqual([
      {
        title: "Key Points",
        level: 2,
        markdown: "First.\n\n### Proposed Solutions\nMore.",
      },
      { title: "Takeaways", level: 2, markdown: "- Remember." },
    ]);
  });

  it("keeps text before the first heading as an untitled section", () => {
    expect(storySections("Intro line.\n\n## Key Points\nFirst.")).toEqual([
      { title: null, level: 0, markdown: "Intro line." },
      { title: "Key Points", level: 2, markdown: "First." },
    ]);
  });

  it("uses the top level present in the body", () => {
    expect(
      storySections("### First\nOne.\n### Second\nTwo.").map((s) => s.title),
    ).toEqual(["First", "Second"]);
  });

  it("shows heading text without markdown", () => {
    expect(storySections("## **Key** `Points`\nFirst.")[0].title).toBe(
      "Key Points",
    );
  });

  it("does not cut at headings inside code blocks", () => {
    expect(storySections("## Code\n```\n## not a heading\n```\n")).toEqual([
      { title: "Code", level: 2, markdown: "```\n## not a heading\n```" },
    ]);
  });

  it("keeps a body without headings whole", () => {
    expect(storySections("Just one paragraph.")).toEqual([
      { title: null, level: 0, markdown: "Just one paragraph." },
    ]);
  });

  it("is empty for an empty body", () => {
    expect(storySections("")).toEqual([]);
  });
});
