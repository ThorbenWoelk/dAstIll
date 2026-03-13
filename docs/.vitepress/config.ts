import { defineConfig } from "vitepress";

export default defineConfig({
  title: "dAstIll Docs",
  description:
    "Architecture, runtime, model, data-flow, and operations documentation for dAstIll.",
  cleanUrls: true,
  lastUpdated: true,
  srcExclude: ["README.md"],
  themeConfig: {
    search: {
      provider: "local",
    },
    nav: [
      { text: "Overview", link: "/" },
      { text: "Architecture", link: "/architecture/overview" },
      { text: "AI & Search", link: "/ai-models" },
      { text: "Operations", link: "/operations/deployment" },
      { text: "UI Tour", link: "/ui-tour" },
    ],
    sidebar: [
      {
        text: "Introduction",
        items: [
          { text: "Overview", link: "/" },
          { text: "Local Development", link: "/local-development" },
          { text: "UI Tour", link: "/ui-tour" },
        ],
      },
      {
        text: "Architecture",
        items: [
          { text: "System Overview", link: "/architecture/overview" },
          { text: "Runtime Topology", link: "/architecture/runtime-topology" },
          { text: "Frontend and API", link: "/architecture/frontend-and-api" },
          { text: "Data Model", link: "/architecture/data-model" },
        ],
      },
      {
        text: "Pipelines",
        items: [
          { text: "Content Pipeline", link: "/flows/content-pipeline" },
          { text: "Search Indexing", link: "/search-indexing" },
          { text: "AI Models", link: "/ai-models" },
        ],
      },
      {
        text: "Operations",
        items: [{ text: "Deployment", link: "/operations/deployment" }],
      },
    ],
    socialLinks: [
      { icon: "github", link: "https://github.com/ThorbenWoelk/dAstIll" },
    ],
    outline: {
      level: [2, 3],
    },
    docFooter: {
      prev: "Previous",
      next: "Next",
    },
  },
});
