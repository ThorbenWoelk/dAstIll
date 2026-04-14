import { defineConfig } from "vitepress";

export default defineConfig({
  title: "dAstIll Docs",
  description:
    "Architecture, runtime, model, data-flow, and operations documentation for dAstIll.",
  cleanUrls: true,
  lastUpdated: false,
  srcExclude: ["README.md"],
  head: [["link", { rel: "icon", href: "/favicon.png" }]],
  themeConfig: {
    search: {
      provider: "local",
    },
    nav: [
      { text: "Overview", link: "/" },
      { text: "Architecture", link: "/architecture/overview" },
      { text: "AI & Search", link: "/pipelines/ai-models" },
      { text: "Operations", link: "/operations/deployment" },
    ],
    sidebar: [
      {
        text: "Introduction",
        items: [
          { text: "Overview", link: "/" },
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
          { text: "Content Pipeline", link: "/pipelines/content-pipeline" },
          { text: "Search Indexing", link: "/pipelines/search-indexing" },
          { text: "AI Models", link: "/pipelines/ai-models" },
        ],
      },
      {
        text: "Operations",
        items: [
          { text: "Deployment", link: "/operations/deployment" },
          { text: "Local Development", link: "/operations/local-development" },
          { text: "Tauri Android", link: "/operations/mobile-tauri" },
        ],
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
