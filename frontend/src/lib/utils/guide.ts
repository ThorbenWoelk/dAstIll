export function resolveGuideStepFromUrl(
  url: URL,
  stepCount: number,
): number | null {
  const guideParam = url.searchParams.get("guide");
  if (guideParam === null) return null;

  const parsed = Number.parseInt(guideParam, 10);
  if (Number.isNaN(parsed) || parsed < 0 || parsed >= stepCount) {
    return null;
  }

  return parsed;
}

export function writeGuideStepToUrl(step: number | null) {
  if (typeof window === "undefined") return;

  const url = new URL(window.location.href);
  if (step === null) {
    url.searchParams.delete("guide");
  } else {
    url.searchParams.set("guide", String(step));
  }

  window.history.replaceState({}, "", url);
}
