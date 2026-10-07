import { describe, expect, it } from "bun:test";

import { toPlainText } from "../src/lib/edition/markdown";
import { firstSentence, splitSummary } from "../src/lib/edition/summary";

const STANDARD = `## At a glance
- First takeaway
- Second takeaway

## Overview
The host explains **why** small teams ship calmer software.

## Key Points
- **Handoffs**: every handoff adds a queue.

## Takeaways
- Ship something small every week.
`;

describe("splitSummary", () => {
  it("turns the standard summary into standfirst, glance box and body", () => {
    const parts = splitSummary(STANDARD);
    expect(parts.standfirst).toBe(
      "The host explains why small teams ship calmer software.",
    );
    expect(parts.glance).toBe("- First takeaway\n- Second takeaway");
    expect(parts.body).toContain("## Key Points");
    expect(parts.body).toContain("## Takeaways");
    expect(parts.body).not.toContain("At a glance");
    expect(parts.body).not.toContain("## Overview");
  });

  it("keeps a long overview in the body and leads with its first sentence", () => {
    const long = `First sentence of a long overview. ${"More detail. ".repeat(40)}`;
    const parts = splitSummary(`## Overview\n${long}\n\n## Key Points\n- One`);
    expect(parts.standfirst).toBe("First sentence of a long overview.");
    expect(parts.body).toContain("## Overview");
  });

  it("renders summaries in other shapes whole as the body", () => {
    const freeform = "Just a paragraph.\n\n### A heading\nMore text.";
    expect(splitSummary(freeform)).toEqual({
      standfirst: "",
      glance: "",
      body: freeform,
    });
  });

  it("accepts the older Brief Overview heading and Windows line endings", () => {
    const parts = splitSummary("## Brief Overview\r\nShort.\r\n");
    expect(parts.standfirst).toBe("Short.");
    expect(parts.body).toBe("");
  });

  it("treats TL;DR and its variants as the glance box", () => {
    for (const heading of [
      "TL;DR",
      "TLDR",
      "tl;dr:",
      "Summary at a glance",
      "Key takeaways at a glance",
    ]) {
      const parts = splitSummary(
        `## ${heading}\n- Point\n\n## Key Points\n- Detail`,
      );
      expect(parts.glance).toBe("- Point");
      expect(parts.body).toBe("## Key Points\n- Detail");
    }
  });

  it("matches headings at levels 1 to 3 with markup, colons, emoji and German names", () => {
    const parts = splitSummary(
      [
        "# **Auf einen Blick:**",
        "- Punkt",
        "",
        "### 📝 Überblick",
        "Kurz gesagt.",
        "",
        "## Kernpunkte",
        "- Detail",
        "",
        "## Fazit",
        "- Merken",
      ].join("\n"),
    );
    expect(parts.glance).toBe("- Punkt");
    expect(parts.standfirst).toBe("Kurz gesagt.");
    expect(parts.body).toBe("## Kernpunkte\n- Detail\n\n## Fazit\n- Merken");
  });

  it("ends the glance box at the next heading of any level", () => {
    const parts = splitSummary(
      "## At a glance\n- Point\n\n### Overview\nShort.\n\n### Details\n- Kept",
    );
    expect(parts.glance).toBe("- Point");
    expect(parts.standfirst).toBe("Short.");
    expect(parts.body).toBe("### Details\n- Kept");
  });

  it("keeps unknown subheadings in place inside Key Points", () => {
    const parts = splitSummary(
      "## Overview\nShort.\n\n## Key Points\n### Proposed Solutions\n- A\n### Overview\n- B\n\n## Takeaways\n- C",
    );
    expect(parts.body).toBe(
      "## Key Points\n### Proposed Solutions\n- A\n### Overview\n- B\n\n## Takeaways\n- C",
    );
  });

  it("keeps a closing # that is part of the heading text", () => {
    const parts = splitSummary("## Overview\nShort.\n\n## Why C#\nBecause.");
    expect(parts.body).toBe("## Why C#\nBecause.");
    expect(splitSummary("## Key Points ##\n- A").body).toBe(
      "## Key Points\n- A",
    );
  });

  it("ignores heading-like lines inside code blocks", () => {
    const code = "```bash\n## not a heading\necho hi\n```";
    const parts = splitSummary(`## Key Points\n${code}\n\n## Takeaways\n- A`);
    expect(parts.body).toBe(`## Key Points\n${code}\n\n## Takeaways\n- A`);
  });

  it("drops model reasoning before the first section heading", () => {
    const parts = splitSummary(
      `Let me analyze this transcript. It covers two ideas.\n\n${STANDARD}`,
    );
    expect(parts.glance).toBe("- First takeaway\n- Second takeaway");
    expect(parts.body).not.toContain("Let me analyze");
    expect(parts.body.startsWith("## Key Points")).toBe(true);
  });

  it("keeps only the last section set when the model wrote a draft first", () => {
    const draft =
      "The task is to summarize.\n\n## At a glance\n- bullets (3-7)\n\n## Overview\n2-3 sentences\n\n## Key Points\nCover each idea.\n\n## Takeaways\n2-3 takeaways.\nLet me write it.</think>";
    const parts = splitSummary(`${draft}${STANDARD}`);
    expect(parts.glance).toBe("- First takeaway\n- Second takeaway");
    expect(parts.standfirst).toBe(
      "The host explains why small teams ship calmer software.",
    );
    expect(parts.body).not.toContain("Cover each idea");
    expect(parts.body).not.toContain("think");
  });

  it("keeps a TL;DR written at the end as the glance box", () => {
    const parts = splitSummary(
      "## Overview\nShort.\n\n## Key Points\n- A\n\n## TL;DR\n- Gist",
    );
    expect(parts.standfirst).toBe("Short.");
    expect(parts.glance).toBe("- Gist");
    expect(parts.body).toBe("## Key Points\n- A");
  });

  it("removes think blocks, stray think tags, chat lines and a wrapping fence", () => {
    const wrapped = `<think>\nPlanning the answer.\n</think>\nHere is the summary:\n\n\`\`\`markdown\n${STANDARD}\`\`\`\n`;
    const parts = splitSummary(wrapped);
    expect(parts.glance).toBe("- First takeaway\n- Second takeaway");
    expect(parts.body).not.toContain("```");
    expect(parts.body).not.toContain("Planning");

    const stray = splitSummary(
      "## Key Points\n- A closing point.</think>## Takeaways\n- Remember",
    );
    expect(stray.body).toBe(
      "## Key Points\n- A closing point.\n## Takeaways\n- Remember",
    );

    const sure = splitSummary("Sure, here you go.\nJust a paragraph.");
    expect(sure.body).toBe("Just a paragraph.");
  });

  it("does not cut the standfirst at abbreviations or initials", () => {
    const tail = " More detail follows here.".repeat(20);
    const parts = splitSummary(
      `## Overview\nIn this video, Dr. Jane R. Smith and Mr. Lee compare U.S. and E.U. rules, e.g. tariffs vs. quotas.${tail}`,
    );
    expect(parts.standfirst).toBe(
      "In this video, Dr. Jane R. Smith and Mr. Lee compare U.S. and E.U. rules, e.g. tariffs vs. quotas.",
    );
  });

  it("cuts a long overview without a sentence end at a word", () => {
    const overview = "word ".repeat(120).trim();
    const parts = splitSummary(
      `## Overview\n${overview}\n\n## Key Points\n- A`,
    );
    expect(parts.standfirst.endsWith("word…")).toBe(true);
    expect(parts.standfirst.length).toBeLessThanOrEqual(241);
    expect(parts.body).toContain(overview);
  });

  it("builds the standfirst from the words a reader sees", () => {
    const parts = splitSummary(
      "## Overview\nR&amp;D at a \\*small\\* lab: `snake_case` names, 5 * 3 = 15, see [the docs](https://x.y).\n\n### Context\nLater.",
    );
    expect(parts.standfirst).toBe(
      "R&D at a *small* lab: snake_case names, 5 * 3 = 15, see the docs.",
    );
    expect(parts.body).toBe("### Context\nLater.");
  });
});

describe("firstSentence", () => {
  it("needs a new sentence to start after the full stop", () => {
    expect(firstSentence("Version 2.5 ships today. It is fast.")).toBe(
      "Version 2.5 ships today.",
    );
    expect(firstSentence('He said "stop." Then he left.')).toBe(
      'He said "stop."',
    );
    expect(firstSentence("A list of apps, e.g. mail. and more")).toBeNull();
    expect(firstSentence("Ends here.")).toBe("Ends here.");
  });
});

describe("toPlainText", () => {
  it("removes inline markdown and collapses whitespace", () => {
    expect(
      toPlainText("A [link](https://x.y) and *em*   `code`\n**bold**"),
    ).toBe("A link and em code bold");
  });

  it("keeps list items apart and raw HTML as text", () => {
    expect(toPlainText("- one\n- two <Tag /> here")).toBe(
      "one two <Tag /> here",
    );
  });

  it("keeps timestamps as text", () => {
    expect(toPlainText("[12:34]: Intro\n[1:02:03] Wrap-up")).toBe(
      "[12:34]: Intro [1:02:03] Wrap-up",
    );
  });
});
