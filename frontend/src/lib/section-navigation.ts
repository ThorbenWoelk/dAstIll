export type SectionNavigationSection = "workspace" | "queue" | "highlights";

export type SectionNavigationItem = {
  section: SectionNavigationSection | "docs";
  label: string;
  href: string;
  active: boolean;
  external: boolean;
};

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
      section: "docs",
      label: "Docs",
      href: docsUrl,
      active: false,
      external: true,
    },
  ];
}
