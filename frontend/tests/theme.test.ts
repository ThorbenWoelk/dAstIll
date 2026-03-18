import { describe, expect, it } from "bun:test";

import {
  DARK_THEME_COLOR,
  LIGHT_THEME_COLOR,
  applyThemeState,
  applyColorScheme,
  parseThemePreference,
  parseThemeMode,
  parseColorScheme,
  resolveModePreference,
  resolveNextThemePreference,
  resolveThemeState,
  DEFAULT_COLOR,
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

describe("parseThemeMode", () => {
  it("accepts light, dark, and system values", () => {
    expect(parseThemeMode("light")).toBe("light");
    expect(parseThemeMode("dark")).toBe("dark");
    expect(parseThemeMode("system")).toBe("system");
  });

  it("defaults to system for unknown values", () => {
    expect(parseThemeMode("auto")).toBe("system");
    expect(parseThemeMode(null)).toBe("system");
    expect(parseThemeMode(undefined)).toBe("system");
  });
});

describe("parseColorScheme", () => {
  it("accepts valid color schemes", () => {
    expect(parseColorScheme("ember")).toBe("ember");
    expect(parseColorScheme("sage")).toBe("sage");
    expect(parseColorScheme("ocean")).toBe("ocean");
    expect(parseColorScheme("sand")).toBe("sand");
    expect(parseColorScheme("plum")).toBe("plum");
  });

  it("defaults to ember for unknown values", () => {
    expect(parseColorScheme("red")).toBe(DEFAULT_COLOR);
    expect(parseColorScheme(null)).toBe(DEFAULT_COLOR);
    expect(parseColorScheme(undefined)).toBe(DEFAULT_COLOR);
  });
});

describe("resolveModePreference", () => {
  it("resolves system mode based on system preference", () => {
    expect(resolveModePreference("system", true)).toBe("dark");
    expect(resolveModePreference("system", false)).toBe("light");
  });

  it("resolves explicit modes directly", () => {
    expect(resolveModePreference("dark", false)).toBe("dark");
    expect(resolveModePreference("light", true)).toBe("light");
  });
});

describe("applyThemeState", () => {
  function createDocumentLike() {
    const toggles: Array<[string, boolean]> = [];
    const metaAttributes: Record<string, string> = {};
    const elAttributes: Record<string, string> = {};
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
        setAttribute(name: string, value: string) {
          elAttributes[name] = value;
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
    return { documentLike, toggles, metaAttributes, elAttributes };
  }

  it("applies the dark class, color scheme, and theme-color meta", () => {
    const { documentLike, toggles, metaAttributes } = createDocumentLike();

    applyThemeState(documentLike, resolveThemeState(null, true));

    expect(toggles).toEqual([["dark", true]]);
    expect(documentLike.documentElement.style.colorScheme).toBe("dark");
    expect(metaAttributes.content).toBe(DARK_THEME_COLOR);
  });

  it("applies data-color attribute via applyColorScheme", () => {
    const { documentLike, elAttributes } = createDocumentLike();

    applyColorScheme(documentLike, "sage");

    expect(elAttributes["data-color"]).toBe("sage");
  });
});
