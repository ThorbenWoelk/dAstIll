import { describe, expect, it } from "vitest";
import { renderMarkdownForChat } from "../src/lib/utils/markdown";

describe("renderMarkdownForChat", () => {
  it("opens markdown links in a new window", () => {
    const html = renderMarkdownForChat("[x](https://example.com/path)");
    expect(html).toContain('target="_blank"');
    expect(html).toContain('rel="noopener noreferrer"');
    expect(html).toContain("https://example.com/path");
  });
});
