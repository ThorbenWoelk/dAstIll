import { describe, expect, it } from "bun:test";

import { resolveCurrentSectionFromPathname } from "../src/lib/mobile-navigation/resolveCurrentSectionFromPathname";
import { shouldCloseDrawerForKey } from "../src/lib/mobile-navigation/drawerKeyboard";

describe("resolveCurrentSectionFromPathname", () => {
  it("maps internal routes to the expected section", () => {
    expect(resolveCurrentSectionFromPathname("/")).toBe("workspace");
    expect(resolveCurrentSectionFromPathname("/download-queue")).toBe("queue");
    expect(resolveCurrentSectionFromPathname("/download-queue/abc")).toBe(
      "queue",
    );
    expect(resolveCurrentSectionFromPathname("/highlights")).toBe("highlights");
    expect(resolveCurrentSectionFromPathname("/chat")).toBe("chat");
  });
});

describe("shouldCloseDrawerForKey", () => {
  it("closes only when drawer is open and key is Escape", () => {
    expect(shouldCloseDrawerForKey(false, "Escape")).toBe(false);
    expect(shouldCloseDrawerForKey(true, "Escape")).toBe(true);
    expect(shouldCloseDrawerForKey(true, "Enter")).toBe(false);
  });
});
