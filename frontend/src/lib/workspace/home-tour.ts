import type { TourStep } from "$lib/components/FeatureGuide.svelte";

export const TAB_STRIP_TOUR = [
  "#workspace-tabs-mobile",
  "#workspace-tabs-desktop",
  "#content-view",
] as const;

export type TourContext = {
  mobileBrowseOpen: boolean;
  selectedVideoId: string | null;
  selectedChannelId: string | null;
  videos: { id: string }[];
  contentMode: string;
  selectVideo: (
    id: string,
    fromUserInteraction?: boolean,
    forceReload?: boolean,
  ) => Promise<void>;
  setMode: (mode: string) => void;
  tick: () => Promise<void>;
};

export function createHomeTourSteps(ctx: TourContext): TourStep[] {
  async function tourPrepareFirstVideoIfNeeded() {
    ctx.mobileBrowseOpen = false;
    await ctx.tick();
    if (
      !ctx.selectedVideoId &&
      ctx.selectedChannelId &&
      ctx.videos.length > 0
    ) {
      await ctx.selectVideo(ctx.videos[0].id, false, false);
    }
    await ctx.tick();
  }

  async function tourPrepareOpenAddChannel() {
    ctx.mobileBrowseOpen = true;
    await ctx.tick();
    document.getElementById("tour-add-channel")?.click();
    await ctx.tick();
    await ctx.tick();
  }

  return [
    {
      selector: "#workspace",
      title: "Welcome to dAstIll",
      body:
        "dAstIll helps you keep up with YouTube without the doom-scrolling. " +
        "It pulls transcripts from your favorite channels and creates AI summaries, " +
        "so you can quickly decide which videos are worth your time. " +
        "Note: This is a showcase app. It's not intended to be a production-ready multi-user application. " +
        "In fact, the business model of YouTube prevent this from ever becoming that. I'm just having fun with it.",
      placement: "right",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
    },
    {
      selector: "#channel-input",
      title: "Add a Channel",
      body: "Paste a URL or handle here to subscribe to a channel. New uploads are tracked automatically.",
      placement: "bottom",
      prepare: () => {
        void tourPrepareOpenAddChannel();
      },
      fallbackSelectors: ["#tour-add-channel", "#tour-library-tools"],
    },
    {
      selector: "#workspace-tabs-mobile",
      title: "Read the Transcript",
      body: "Every video's spoken content is available as a full transcript you can read at your own pace.",
      placement: "bottom",
      prepare: async () => {
        await tourPrepareFirstVideoIfNeeded();
        if (ctx.contentMode !== "transcript") {
          await ctx.setMode("transcript");
        }
      },
      fallbackSelectors: [...TAB_STRIP_TOUR],
    },
    {
      selector: "#workspace-tabs-mobile",
      title: "AI Summary",
      body: "The Summary tab shows the distilled version so you can decide if the full video is still worth watching.",
      placement: "bottom",
      prepare: async () => {
        await tourPrepareFirstVideoIfNeeded();
        if (ctx.contentMode !== "summary") {
          await ctx.setMode("summary");
        }
      },
      fallbackSelectors: [...TAB_STRIP_TOUR],
    },
    {
      selector: '[data-tour-target="nav-chat"]',
      title: "AI Chat",
      body: "Chat with your library. Our agentic RAG-based LLM system let's you ask questions about specific videos and will even do deep research for you.",
      placement: "right",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
      fallbackSelectors: [
        "#nav-chat-link",
        "#mobile-nav-chat-link",
        "#app-section-nav-rail a[href='/chat']",
        "#app-section-nav-mobile a[href='/chat']",
      ],
    },
    {
      selector: "#workspace",
      title: "Other features",
      body: "Search, sort, and filter videos. Set earliest date to sync from and load more videos to go further back in time.",
      placement: "bottom",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
      fallbackSelectors: ["#tour-library-tools"],
    },
    {
      selector: "#mark-read-toggle",
      title: "Mark as read",
      body: "Tip: Use it with the read filter in the library to get that sweet dopamine feeling of progress.",
      placement: "bottom",
      prepare: async () => {
        if (ctx.contentMode === "info" || ctx.contentMode === "highlights") {
          await ctx.setMode("transcript");
        }
        await tourPrepareFirstVideoIfNeeded();
      },
      fallbackSelectors: [
        "#content-actions",
        "#workspace-tabs-mobile",
        "#workspace-tabs-desktop",
        "#content-view",
      ],
    },
    {
      selector: "#workspace-tabs-mobile",
      title: "Your Highlights",
      body: "Found something worth remembering? Select any text in the transcript or summary and save it as a highlight. All your saved passages for this video appear here.",
      placement: "bottom",
      prepare: async () => {
        await tourPrepareFirstVideoIfNeeded();
        if (ctx.contentMode !== "highlights") {
          await ctx.setMode("highlights");
        }
      },
      fallbackSelectors: [...TAB_STRIP_TOUR],
    },
    {
      selector: "#ai-status-pill",
      title: "AI availability",
      body: "This dot beside the logo shows whether the AI backend is reachable for summaries and chat. Reading still works without it.",
      placement: "bottom",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
      fallbackSelectors: [
        "a[aria-label='Go to dAstIll home']",
        "#nav-workspace-link",
        "#mobile-nav-workspace-link",
      ],
    },
    {
      selector: "#guide-trigger",
      title: "Come back to this guide any time",
      body: "Reopen this guide at any time.",
      placement: "right",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
      fallbackSelectors: ["#workspace"],
    },
  ];
}
