import { describe, expect, it } from "bun:test";

import {
  getSectionNavigationItems,
  type SectionNavigationSection,
} from "../src/lib/section-navigation";

describe("getSectionNavigationItems", () => {
  it("marks the active internal section and preserves the docs link", () => {
    const items = getSectionNavigationItems(
      "queue",
      "https://docs.example.com",
    );

    expect(items).toHaveLength(4);
    expect(items.map((item) => item.label)).toEqual([
      "Workspace",
      "Queue",
      "Highlights",
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
