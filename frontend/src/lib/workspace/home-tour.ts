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
  /** When false, the tour skips selecting a video (avoids API calls that require sign-in). */
  isAuthenticated: () => boolean;
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
    if (!ctx.isAuthenticated()) {
      await ctx.tick();
      return;
    }
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
        "Use the workspace to browse followed sources, open a recent item, " +
        "and read the summary or transcript without leaving the main shell. " +
        "On mobile web, the flow is browse first, then read, then jump to queue or chat when you need more depth.",
      placement: "right",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
    },
    {
      selector: "#drawer-source-input",
      title: "Add a source",
      body: "Paste a YouTube URL or handle here to add a source. The add flow stays in the browse sheet so you can keep your place.",
      placement: "bottom",
      prepare: () => {
        void tourPrepareOpenAddChannel();
      },
      fallbackSelectors: ["#tour-add-channel", "#tour-library-tools"],
    },
    {
      selector: "#workspace-tabs-mobile",
      title: "Open the transcript",
      body: "Transcript keeps the full spoken text available when you need detail, want to verify a claim, or want to capture a precise highlight.",
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
      title: "Read the summary",
      body: "Summary is the fastest mobile read. Use it to decide whether the item deserves a deeper transcript pass or can be cleared quickly.",
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
      title: "Open library chat",
      body: "Chat across the same library when you want synthesis instead of one-item reading. Deep research stays in the conversation flow.",
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
      title: "Browse tools",
      body: "Search, filter, change sync depth, and load older items from the same workspace shell instead of opening a separate management page.",
      placement: "bottom",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
      fallbackSelectors: ["#tour-library-tools"],
    },
    {
      selector: "#mark-read-toggle",
      title: "Mark items read",
      body: "Use this to clear items after you finish triaging them. It works best together with the read filter back in browse.",
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
      title: "Save highlights",
      body: "Select text in the transcript or summary and save it here. Highlights turn quick mobile reading into something you can return to later.",
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
      title: "Check AI status",
      body: "This dot beside the logo shows whether summaries and chat are currently reachable. Reading still works even when AI is unavailable.",
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
      title: "Reopen the guide",
      body: "Come back to this walkthrough any time from the workspace.",
      placement: "right",
      prepare: () => {
        ctx.mobileBrowseOpen = true;
      },
      fallbackSelectors: ["#workspace"],
    },
  ];
}
