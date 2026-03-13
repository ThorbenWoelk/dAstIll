import { describe, expect, it } from "bun:test";

import {
  DARK_THEME_COLOR,
  LIGHT_THEME_COLOR,
  applyThemeState,
  parseThemePreference,
  resolveNextThemePreference,
  resolveThemeState,
} from "../src/lib/theme";

describe("parseThemePreference", () => {
  it("accepts explicit light and dark values", () => {
    expect(parseThemePreference("light")).toBe("light");
    expect(parseThemePreference("dark")).toBe("dark");
  });

  it("treats unknown values as unset", () => {
    expect(parseThemePreference("auto")).toBeNull();
    expect(parseThemePreference("")).toBeNull();
    expect(parseThemePreference(null)).toBeNull();
    expect(parseThemePreference(undefined)).toBeNull();
  });
});

describe("resolveThemeState", () => {
  it("uses an explicit dark preference", () => {
    expect(resolveThemeState("dark", false)).toEqual({
      preference: "dark",
      isDark: true,
      colorScheme: "dark",
      themeColor: DARK_THEME_COLOR,
    });
  });

  it("uses an explicit light preference", () => {
    expect(resolveThemeState("light", true)).toEqual({
      preference: "light",
      isDark: false,
      colorScheme: "light",
      themeColor: LIGHT_THEME_COLOR,
    });
  });

  it("falls back to the system preference when unset", () => {
    expect(resolveThemeState(null, true).isDark).toBe(true);
    expect(resolveThemeState(null, false).isDark).toBe(false);
  });
});

describe("resolveNextThemePreference", () => {
  it("toggles between explicit light and dark preferences", () => {
    expect(resolveNextThemePreference(false)).toBe("dark");
    expect(resolveNextThemePreference(true)).toBe("light");
  });
});

describe("applyThemeState", () => {
  it("applies the dark class, color scheme, and theme-color meta", () => {
    const toggles: Array<[string, boolean]> = [];
    const metaAttributes: Record<string, string> = {};
    const documentLike = {
      documentElement: {
        classList: {
          toggle(name: string, force: boolean) {
            toggles.push([name, force]);
          },
        },
        style: {
          colorScheme: "light",
        },
      },
      querySelector(selector: string) {
        expect(selector).toBe('meta[name="theme-color"]');
        return {
          setAttribute(name: string, value: string) {
            metaAttributes[name] = value;
          },
        };
      },
    };

    applyThemeState(documentLike, resolveThemeState(null, true));

    expect(toggles).toEqual([["dark", true]]);
    expect(documentLike.documentElement.style.colorScheme).toBe("dark");
    expect(metaAttributes.content).toBe(DARK_THEME_COLOR);
  });
});
