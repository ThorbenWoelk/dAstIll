/** Shared SVG icon data for section navigation items, used by both the desktop nav rail and the mobile bottom tab bar. */
export function sectionIcon(section: string): {
  viewBox: string;
  paths: string[];
} {
  switch (section) {
    case "workspace":
      return {
        viewBox: "0 0 24 24",
        paths: ["M3 4h6v16H3z", "M10 4h5v16h-5z", "M16 4h5v16h-5z"],
      };
    case "highlights":
      return {
        viewBox: "0 0 24 24",
        paths: [
          "M7 4h10l2 4v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V8z",
          "M9 12h6",
          "M9 16h4",
          "M9 4v4h6V4",
        ],
      };
    case "vocabulary":
      return {
        viewBox: "0 0 24 24",
        paths: ["M4 6h16", "M4 12h10", "M4 18h7", "M18 10l2 2-4 4-2-2z"],
      };
    case "chat":
      return {
        viewBox: "0 0 24 24",
        paths: [
          "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z",
          "M8 9h8",
          "M8 13h5",
        ],
      };
    case "docs":
      return {
        viewBox: "0 0 24 24",
        paths: [
          "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z",
          "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z",
        ],
      };
    default:
      return { viewBox: "0 0 24 24", paths: [] };
  }
}
