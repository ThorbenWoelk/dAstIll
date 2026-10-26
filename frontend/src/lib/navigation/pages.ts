/** The paper's pages. Phones show them as a tab bar along the bottom. */
export type PaperPage = "front-page" | "highlights" | "finished" | "profile";

export const PAPER_PAGES: { id: PaperPage; name: string; href: string }[] = [
  { id: "front-page", name: "Front page", href: "/" },
  { id: "highlights", name: "Highlights", href: "/highlights" },
  { id: "finished", name: "Finished", href: "/finished" },
  { id: "profile", name: "Profile", href: "/profile" },
];

/**
 * The name of the page a "Back" control returns to: the paper page the
 * reader came from, or plain "Back" for anything else.
 */
export function describeWayBack(pathname: string): string {
  const path = pathname.replace(/\/+$/, "") || "/";
  return PAPER_PAGES.find((page) => page.href === path)?.name ?? "Back";
}
