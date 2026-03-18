export const THEME_STORAGE_KEY = "dastill-theme-appearance";
export const COLOR_STORAGE_KEY = "dastill-theme-color";
export const LIGHT_THEME_COLOR = "#f8f1ec";
export const DARK_THEME_COLOR = "#111315";

export type ThemeMode = "light" | "dark" | "system";
export type ThemePreference = "light" | "dark";
export type ColorScheme = "ember" | "sage" | "ocean" | "sand" | "plum";

export const COLOR_SCHEMES: {
  id: ColorScheme;
  label: string;
  swatch: string;
}[] = [
  { id: "ember", label: "Ember", swatch: "#d33c2a" },
  { id: "sage", label: "Sage", swatch: "#4a8a5c" },
  { id: "ocean", label: "Ocean", swatch: "#2a7ab5" },
  { id: "sand", label: "Sand", swatch: "#a68a5b" },
  { id: "plum", label: "Plum", swatch: "#8b5cb4" },
];

export const DEFAULT_COLOR: ColorScheme = "ember";

export type ThemeState = {
  preference: ThemePreference | null;
  isDark: boolean;
  colorScheme: "light" | "dark";
  themeColor: string;
};

type ThemeMetaLike = {
  setAttribute(name: string, value: string): void;
};

type ThemeDocumentLike = {
  documentElement: {
    classList: {
      toggle(name: string, force?: boolean): void;
    };
    style: {
      colorScheme: string;
    };
    setAttribute(name: string, value: string): void;
  };
  querySelector(selector: 'meta[name="theme-color"]'): ThemeMetaLike | null;
};

export function parseThemePreference(
  value: string | null | undefined,
): ThemePreference | null {
  if (value === "light" || value === "dark") {
    return value;
  }
  return null;
}

export function parseThemeMode(value: string | null | undefined): ThemeMode {
  if (value === "light" || value === "dark" || value === "system") {
    return value;
  }
  return "system";
}

export function parseColorScheme(
  value: string | null | undefined,
): ColorScheme {
  if (
    value === "ember" ||
    value === "sage" ||
    value === "ocean" ||
    value === "sand" ||
    value === "plum"
  ) {
    return value;
  }
  return DEFAULT_COLOR;
}

export function readThemePreference(
  storage: Pick<Storage, "getItem">,
  key = THEME_STORAGE_KEY,
): ThemePreference | null {
  return parseThemePreference(storage.getItem(key));
}

export function readThemeMode(
  storage: Pick<Storage, "getItem">,
  key = THEME_STORAGE_KEY,
): ThemeMode {
  return parseThemeMode(storage.getItem(key));
}

export function readColorScheme(
  storage: Pick<Storage, "getItem">,
  key = COLOR_STORAGE_KEY,
): ColorScheme {
  return parseColorScheme(storage.getItem(key));
}

export function writeThemePreference(
  storage: Pick<Storage, "setItem">,
  preference: ThemePreference,
  key = THEME_STORAGE_KEY,
) {
  storage.setItem(key, preference);
}

export function writeThemeMode(
  storage: Pick<Storage, "setItem">,
  mode: ThemeMode,
  key = THEME_STORAGE_KEY,
) {
  storage.setItem(key, mode);
}

export function writeColorScheme(
  storage: Pick<Storage, "setItem">,
  color: ColorScheme,
  key = COLOR_STORAGE_KEY,
) {
  storage.setItem(key, color);
}

export function resolveThemeState(
  preference: ThemePreference | null,
  systemPrefersDark: boolean,
): ThemeState {
  const isDark =
    preference === "dark" || (preference === null && systemPrefersDark);

  return {
    preference,
    isDark,
    colorScheme: isDark ? "dark" : "light",
    themeColor: isDark ? DARK_THEME_COLOR : LIGHT_THEME_COLOR,
  };
}

export function resolveModePreference(
  mode: ThemeMode,
  systemPrefersDark: boolean,
): ThemePreference | null {
  if (mode === "system") return systemPrefersDark ? "dark" : "light";
  return mode;
}

export function resolveNextThemePreference(isDark: boolean): ThemePreference {
  return isDark ? "light" : "dark";
}

export function applyThemeState(
  documentLike: ThemeDocumentLike,
  state: ThemeState,
) {
  documentLike.documentElement.classList.toggle("dark", state.isDark);
  documentLike.documentElement.style.colorScheme = state.colorScheme;
  documentLike
    .querySelector('meta[name="theme-color"]')
    ?.setAttribute("content", state.themeColor);
}

export function applyColorScheme(
  documentLike: ThemeDocumentLike,
  color: ColorScheme,
) {
  documentLike.documentElement.setAttribute("data-color", color);
}
