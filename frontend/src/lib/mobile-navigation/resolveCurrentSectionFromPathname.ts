import type { SectionNavigationSection } from "$lib/section-navigation";

export function resolveCurrentSectionFromPathname(
  pathname: string,
): SectionNavigationSection {
  if (pathname.startsWith("/download-queue")) return "queue";
  if (pathname.startsWith("/highlights")) return "highlights";
  if (pathname.startsWith("/vocabulary")) return "vocabulary";
  if (pathname.startsWith("/chat")) return "chat";
  return "workspace";
}
