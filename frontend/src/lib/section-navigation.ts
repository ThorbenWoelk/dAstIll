export type SectionNavigationSection =
  | "workspace"
  | "queue"
  | "highlights"
  | "vocabulary"
  | "chat";

export type SectionNavigationItem = {
  section: SectionNavigationSection | "docs";
  label: string;
  href: string;
  active: boolean;
  external: boolean;
};

/** Letter shown next to a section row after pressing G (go navigation chord). */
export function goHintKeyForSection(
  section: SectionNavigationSection | "docs",
): string {
  const keys: Record<SectionNavigationSection | "docs", string> = {
    workspace: "W",
    queue: "Q",
    highlights: "H",
    vocabulary: "V",
    chat: "C",
    docs: "D",
  };
  return keys[section];
}

export function getSectionNavigationItems(
  currentSection: SectionNavigationSection,
  docsUrl: string,
): SectionNavigationItem[] {
  return [
    {
      section: "workspace",
      label: "Workspace",
      href: "/",
      active: currentSection === "workspace",
      external: false,
    },
    {
      section: "queue",
      label: "Queue",
      href: "/download-queue",
      active: currentSection === "queue",
      external: false,
    },
    {
      section: "highlights",
      label: "Highlights",
      href: "/highlights",
      active: currentSection === "highlights",
      external: false,
    },
    {
      section: "vocabulary",
      label: "Vocabulary",
      href: "/vocabulary",
      active: currentSection === "vocabulary",
      external: false,
    },
    {
      section: "chat",
      label: "Chat",
      href: "/chat",
      active: currentSection === "chat",
      external: false,
    },
    {
      section: "docs",
      label: "Docs",
      href: docsUrl,
      active: false,
      external: true,
    },
  ];
}
