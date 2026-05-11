export type SectionNavigationSection =
  | "workspace"
  | "highlights"
  | "vocabulary"
  | "chat";

export type AppNavigationSection = SectionNavigationSection | "docs";

type SectionNavigationDescriptor = {
  section: AppNavigationSection;
  label: string;
  href: string | null;
  external: boolean;
};

export type SectionNavigationItem = {
  section: AppNavigationSection;
  label: string;
  href: string;
  active: boolean;
  external: boolean;
};

export const SECTION_NAVIGATION_ITEMS: readonly SectionNavigationDescriptor[] =
  [
    {
      section: "workspace",
      label: "Workspace",
      href: "/",
      external: false,
    },
    {
      section: "highlights",
      label: "Highlights",
      href: "/highlights",
      external: false,
    },
    {
      section: "vocabulary",
      label: "Vocabulary",
      href: "/vocabulary",
      external: false,
    },
    {
      section: "chat",
      label: "Chat",
      href: "/chat",
      external: false,
    },
    {
      section: "docs",
      label: "Docs",
      href: null,
      external: true,
    },
  ] as const;

/** Shortcut number (1-6) shown next to a section row after pressing Cmd. */
export function goHintKeyForSection(section: AppNavigationSection): string {
  const index = SECTION_NAVIGATION_ITEMS.findIndex(
    (item) => item.section === section,
  );
  return index === -1 ? "" : String(index + 1);
}

export function getSectionNavigationItems(
  currentSection: SectionNavigationSection,
  docsUrl: string,
): SectionNavigationItem[] {
  return SECTION_NAVIGATION_ITEMS.map((item) => ({
    section: item.section,
    label: item.label,
    href: item.href ?? docsUrl,
    active: item.section === currentSection,
    external: item.external,
  }));
}
