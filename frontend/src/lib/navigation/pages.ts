/** The paper's pages. Phones show them as a tab bar along the bottom. */
export type PaperPage = "front-page" | "highlights" | "finished" | "sections";

export const PAPER_PAGES: { id: PaperPage; name: string; href: string }[] = [
  { id: "front-page", name: "Front page", href: "/" },
  { id: "highlights", name: "Highlights", href: "/highlights" },
  { id: "finished", name: "Finished", href: "/finished" },
  { id: "sections", name: "Sections", href: "/sections" },
];

/**
 * The name of the page a "Back" control returns to: the paper page the
 * reader came from, or plain "Back" for anything else.
 */
export function describeWayBack(pathname: string): string {
  const path = pathname.replace(/\/+$/, "") || "/";
  return PAPER_PAGES.find((page) => page.href === path)?.name ?? "Back";
}
