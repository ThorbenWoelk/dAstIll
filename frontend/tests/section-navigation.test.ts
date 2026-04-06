import { describe, expect, it } from "bun:test";

import { resolveCurrentSectionFromPathname } from "../src/lib/mobile-navigation/resolveCurrentSectionFromPathname";
import {
  getSectionNavigationItems,
  goHintKeyForSection,
  type SectionNavigationSection,
} from "../src/lib/section-navigation";

describe("resolveCurrentSectionFromPathname", () => {
  it("maps primary routes for section highlighting and keyboard hints", () => {
    expect(resolveCurrentSectionFromPathname("/")).toBe("workspace");
    expect(resolveCurrentSectionFromPathname("/download-queue")).toBe("queue");
    expect(resolveCurrentSectionFromPathname("/download-queue/")).toBe("queue");
    expect(resolveCurrentSectionFromPathname("/highlights")).toBe("highlights");
    expect(resolveCurrentSectionFromPathname("/vocabulary")).toBe("vocabulary");
    expect(resolveCurrentSectionFromPathname("/chat")).toBe("chat");
    expect(resolveCurrentSectionFromPathname("/chat/c1")).toBe("chat");
  });
});

describe("goHintKeyForSection", () => {
  it("maps each section to its go chord number", () => {
    expect(goHintKeyForSection("workspace")).toBe("1");
    expect(goHintKeyForSection("queue")).toBe("2");
    expect(goHintKeyForSection("highlights")).toBe("3");
    expect(goHintKeyForSection("vocabulary")).toBe("4");
    expect(goHintKeyForSection("chat")).toBe("5");
    expect(goHintKeyForSection("docs")).toBe("6");
  });
});

describe("getSectionNavigationItems", () => {
  it("marks the active internal section and preserves the docs link", () => {
    const items = getSectionNavigationItems(
      "queue",
      "https://docs.example.com",
    );

    expect(items).toHaveLength(6);
    expect(items.map((item) => item.label)).toEqual([
      "Workspace",
      "Queue",
      "Highlights",
      "Vocabulary",
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
      "vocabulary",
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
