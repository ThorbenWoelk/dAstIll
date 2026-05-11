/** Shared SVG icon data for section navigation items, used by both the desktop nav rail and the mobile bottom tab bar. Paths mirror the Solar linear icon set. */
export function sectionIcon(section: string): {
  viewBox: string;
  paths: string[];
} {
  switch (section) {
    case "workspace":
      return {
        viewBox: "0 0 24 24",
        paths: [
          "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
        ],
      };
    case "highlights":
      return {
        viewBox: "0 0 24 24",
        paths: ["M6 3h12v18l-6-4-6 4z"],
      };
    case "vocabulary":
      return {
        viewBox: "0 0 24 24",
        paths: [
          "M3 4h14a4 4 0 0 1 4 4v12H7a4 4 0 0 1-4-4z",
          "M7 9h10",
          "M7 13h8",
        ],
      };
    case "chat":
      return {
        viewBox: "0 0 24 24",
        paths: [
          "M4 12a8 8 0 0 1 16 0c0 4.418-3.582 8-8 8-1.1 0-2.15-.22-3.1-.62L4 21l1.62-4.9A8 8 0 0 1 4 12Z",
          "M9 12h.01M12 12h.01M15 12h.01",
        ],
      };
    case "docs":
      return {
        viewBox: "0 0 24 24",
        paths: [
          "M7 3h8l4 4v12a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2Z",
          "M14 3v5h5",
          "M9 13h6M9 17h4",
        ],
      };
    default:
      return { viewBox: "0 0 24 24", paths: [] };
  }
}
