import { defineConfig } from "vitepress";

export default defineConfig({
  title: "dAstIll Docs",
  description:
    "Architecture, features, runtime, model, data-flow, and operations documentation for dAstIll.",
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
      { text: "Features", link: "/features/summarization" },
      { text: "Operations", link: "/operations/deployment" },
      { text: "Security", link: "/security/" },
    ],
    sidebar: [
      {
        text: "Introduction",
        items: [
          { text: "Overview", link: "/" },
          { text: "Disclaimer", link: "/disclaimer" },
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
        text: "Features",
        items: [
          { text: "Summarization", link: "/features/summarization" },
          { text: "Search", link: "/features/search" },
          { text: "Chat", link: "/features/chat" },
          { text: "TTS", link: "/features/tts" },
          { text: "Mini Reader", link: "/features/mini-reader" },
        ],
      },
      {
        text: "Pipelines",
        items: [
          { text: "Content Pipeline", link: "/pipelines/content-pipeline" },
          { text: "AI Models", link: "/pipelines/ai-models" },
        ],
      },
      {
        text: "Operations",
        items: [
          { text: "Deployment", link: "/operations/deployment" },
          { text: "Runtime Limits", link: "/operations/runtime-limits" },
          { text: "Local Development", link: "/operations/local-development" },
          { text: "Android Operations", link: "/operations/mobile-tauri" },
        ],
      },
      {
        text: "Security",
        items: [
          { text: "Security Overview", link: "/security/" },
          { text: "OWASP ASI Status", link: "/security/owasp-asi-status" },
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
