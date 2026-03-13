import { describe, expect, it } from "bun:test";
import type {
  CreateHighlightRequest,
  Highlight,
  HighlightChannelGroup,
} from "../src/lib/types";
import {
  buildHighlightDraft,
  buildOptimisticHighlight,
  mergeHighlightIntoList,
  removeHighlightFromGroups,
  reconcileOptimisticHighlight,
  resolveHighlightRanges,
  resolveTooltipPosition,
} from "../src/lib/utils/highlights";

describe("buildHighlightDraft", () => {
  it("derives highlight text plus surrounding context from a selected range", () => {
    const fullText = "Alpha beta gamma delta epsilon";

    expect(buildHighlightDraft(fullText, "transcript", 6, 16, 5)).toEqual({
      source: "transcript",
      text: "beta gamma",
      prefix_context: "lpha ",
      suffix_context: " delt",
    });
  });
});

describe("resolveHighlightRanges", () => {
  it("uses stored context to choose the intended repeated phrase", () => {
    const fullText =
      "Intro insight appears here. Later, the same insight appears again.";
    const highlights: Highlight[] = [
      {
        id: 1,
        video_id: "vid-1",
        source: "transcript",
        text: "insight appears",
        prefix_context: "Later, the same ",
        suffix_context: " again.",
        created_at: "2026-03-12T20:00:00.000Z",
      },
    ];

    expect(resolveHighlightRanges(fullText, highlights)).toEqual([
      {
        highlightId: 1,
        start: 44,
        end: 59,
      },
    ]);
  });

  it("skips overlapping matches after the first accepted range", () => {
    const fullText = "alpha beta gamma";
    const highlights: Highlight[] = [
      {
        id: 1,
        video_id: "vid-1",
        source: "summary",
        text: "alpha beta",
        prefix_context: "",
        suffix_context: " gamma",
        created_at: "2026-03-12T20:00:00.000Z",
      },
      {
        id: 2,
        video_id: "vid-1",
        source: "summary",
        text: "beta gamma",
        prefix_context: "alpha ",
        suffix_context: "",
        created_at: "2026-03-12T20:00:01.000Z",
      },
    ];

    expect(resolveHighlightRanges(fullText, highlights)).toEqual([
      {
        highlightId: 1,
        start: 0,
        end: 10,
      },
    ]);
  });
});

describe("resolveTooltipPosition", () => {
  it("converts viewport selection coordinates into container-relative tooltip coordinates", () => {
    expect(
      resolveTooltipPosition(
        {
          left: 720,
          top: 320,
          width: 120,
        },
        {
          left: 560,
          top: 180,
          width: 980,
        },
      ),
    ).toEqual({
      left: 220,
      top: 94,
    });
  });
});

describe("removeHighlightFromGroups", () => {
  it("removes the highlight and prunes empty videos and channels", () => {
    const groups: HighlightChannelGroup[] = [
      {
        channel_id: "channel-1",
        channel_name: "Channel 1",
        channel_thumbnail_url: null,
        videos: [
          {
            video_id: "video-1",
            title: "Video 1",
            thumbnail_url: null,
            published_at: "2026-03-10T00:00:00.000Z",
            highlights: [
              {
                id: 1,
                video_id: "video-1",
                source: "summary",
                text: "First",
                prefix_context: "",
                suffix_context: "",
                created_at: "2026-03-12T20:00:00.000Z",
              },
            ],
          },
        ],
      },
      {
        channel_id: "channel-2",
        channel_name: "Channel 2",
        channel_thumbnail_url: null,
        videos: [
          {
            video_id: "video-2",
            title: "Video 2",
            thumbnail_url: null,
            published_at: "2026-03-11T00:00:00.000Z",
            highlights: [
              {
                id: 2,
                video_id: "video-2",
                source: "transcript",
                text: "Keep me",
                prefix_context: "",
                suffix_context: "",
                created_at: "2026-03-12T20:00:01.000Z",
              },
            ],
          },
        ],
      },
    ];

    expect(removeHighlightFromGroups(groups, 1)).toEqual([
      {
        channel_id: "channel-2",
        channel_name: "Channel 2",
        channel_thumbnail_url: null,
        videos: [
          {
            video_id: "video-2",
            title: "Video 2",
            thumbnail_url: null,
            published_at: "2026-03-11T00:00:00.000Z",
            highlights: [
              {
                id: 2,
                video_id: "video-2",
                source: "transcript",
                text: "Keep me",
                prefix_context: "",
                suffix_context: "",
                created_at: "2026-03-12T20:00:01.000Z",
              },
            ],
          },
        ],
      },
    ]);
  });
});

describe("optimistic highlight helpers", () => {
  it("builds an optimistic highlight from the pending create payload", () => {
    const payload: CreateHighlightRequest = {
      source: "summary",
      text: "Ship the small change first",
      prefix_context: "Before ",
      suffix_context: " after",
    };

    expect(
      buildOptimisticHighlight(
        "vid-7",
        payload,
        -1,
        "2026-03-12T22:15:00.000Z",
      ),
    ).toEqual({
      id: -1,
      video_id: "vid-7",
      source: "summary",
      text: "Ship the small change first",
      prefix_context: "Before ",
      suffix_context: " after",
      created_at: "2026-03-12T22:15:00.000Z",
    });
  });

  it("merges highlights newest-first and replaces matching ids", () => {
    const current: Highlight[] = [
      {
        id: 1,
        video_id: "vid-1",
        source: "transcript",
        text: "Older saved item",
        prefix_context: "",
        suffix_context: "",
        created_at: "2026-03-11T12:00:00.000Z",
      },
    ];

    const optimistic = buildOptimisticHighlight(
      "vid-1",
      {
        source: "transcript",
        text: "New item",
        prefix_context: "",
        suffix_context: "",
      },
      -1,
      "2026-03-12T12:00:00.000Z",
    );

    expect(mergeHighlightIntoList(current, optimistic)).toEqual([
      optimistic,
      current[0],
    ]);
  });

  it("reconciles an optimistic highlight with the confirmed server record", () => {
    const optimistic = buildOptimisticHighlight(
      "vid-1",
      {
        source: "transcript",
        text: "Important point",
        prefix_context: "",
        suffix_context: "",
      },
      -1,
      "2026-03-12T12:00:00.000Z",
    );
    const existing: Highlight = {
      id: 1,
      video_id: "vid-1",
      source: "summary",
      text: "Earlier note",
      prefix_context: "",
      suffix_context: "",
      created_at: "2026-03-11T12:00:00.000Z",
    };
    const confirmed: Highlight = {
      id: 7,
      video_id: "vid-1",
      source: "transcript",
      text: "Important point",
      prefix_context: "",
      suffix_context: "",
      created_at: "2026-03-12T12:00:01.000Z",
    };

    expect(
      reconcileOptimisticHighlight(
        [optimistic, existing],
        optimistic.id,
        confirmed,
      ),
    ).toEqual([confirmed, existing]);
  });
});
