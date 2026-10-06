import { describe, expect, it } from "bun:test";

import { splitSummary, toPlainText } from "../src/lib/edition/summary";

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
});

describe("toPlainText", () => {
  it("removes inline markdown and collapses whitespace", () => {
    expect(
      toPlainText("A [link](https://x.y) and *em*   `code`\n**bold**"),
    ).toBe("A link and em code bold");
  });
});
