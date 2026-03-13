export const THEME_STORAGE_KEY = "dastill-theme-appearance";
export const LIGHT_THEME_COLOR = "#f8f1ec";
export const DARK_THEME_COLOR = "#111315";

export type ThemePreference = "light" | "dark";

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

export function readThemePreference(
  storage: Pick<Storage, "getItem">,
  key = THEME_STORAGE_KEY,
): ThemePreference | null {
  return parseThemePreference(storage.getItem(key));
}

export function writeThemePreference(
  storage: Pick<Storage, "setItem">,
  preference: ThemePreference,
  key = THEME_STORAGE_KEY,
) {
  storage.setItem(key, preference);
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
