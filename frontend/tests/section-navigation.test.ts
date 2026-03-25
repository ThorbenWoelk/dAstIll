import { describe, expect, it } from "bun:test";

import {
  getSectionNavigationItems,
  goHintKeyForSection,
  type SectionNavigationSection,
} from "../src/lib/section-navigation";

describe("goHintKeyForSection", () => {
  it("maps each section to its go chord letter", () => {
    expect(goHintKeyForSection("workspace")).toBe("W");
    expect(goHintKeyForSection("queue")).toBe("Q");
    expect(goHintKeyForSection("highlights")).toBe("H");
    expect(goHintKeyForSection("chat")).toBe("C");
    expect(goHintKeyForSection("docs")).toBe("D");
  });
});

describe("getSectionNavigationItems", () => {
  it("marks the active internal section and preserves the docs link", () => {
    const items = getSectionNavigationItems(
      "queue",
      "https://docs.example.com",
    );

    expect(items).toHaveLength(5);
    expect(items.map((item) => item.label)).toEqual([
      "Workspace",
      "Queue",
      "Highlights",
      "Chat",
      "Docs",
    ]);
    expect(items.find((item) => item.section === "queue")?.active).toBe(true);
    expect(items.find((item) => item.section === "workspace")?.active).toBe(
      false,
    );
    expect(items.at(-1)).toEqual({
      section: "docs",
      label: "Docs",
      href: "https://docs.example.com",
      active: false,
      external: true,
    });
  });

  it("returns exactly one active section for each internal route", () => {
    const sections: SectionNavigationSection[] = [
      "workspace",
      "queue",
      "highlights",
      "chat",
    ];

    for (const section of sections) {
      const items = getSectionNavigationItems(
        section,
        "https://docs.example.com",
      );
      expect(items.filter((item) => item.active)).toEqual([
        expect.objectContaining({ section, active: true }),
      ]);
    }
  });
});
