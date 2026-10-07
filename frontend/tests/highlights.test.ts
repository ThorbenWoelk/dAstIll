import { describe, expect, it } from "bun:test";

import type { Highlight, HighlightChannelGroup } from "../src/lib/api";
import {
  buildHighlightDraft,
  placeHighlights,
} from "../src/lib/highlights/anchoring";
import {
  countHighlights,
  filterHighlightedStories,
  listHighlightedStories,
  withoutHighlight,
} from "../src/lib/highlights/collection";

function highlight(
  id: string,
  text: string,
  prefix_context = "",
  suffix_context = "",
): Highlight {
  return {
    id,
    video_id: "v1",
    source: "summary",
    text,
    prefix_context,
    suffix_context,
    created_at: "2026-10-07T08:00:00Z",
  };
}

describe("buildHighlightDraft", () => {
  it("trims the selection and keeps context on both sides", () => {
    const text = "Sleep pressure builds all day. Adenosine is the signal.";
    const start = text.indexOf("Adenosine") - 1;
    const end = text.length;
    expect(buildHighlightDraft(text, start, end)).toEqual({
      source: "summary",
      text: "Adenosine is the signal.",
      prefix_context: "Sleep pressure builds all day. ",
      suffix_context: "",
    });
  });

  it("limits context to 80 characters", () => {
    const text = `${"a".repeat(200)}HIT${"b".repeat(200)}`;
    const draft = buildHighlightDraft(text, 200, 203)!;
    expect(draft.prefix_context).toBe("a".repeat(80));
    expect(draft.suffix_context).toBe("b".repeat(80));
  });

  it("ignores a selection of only whitespace", () => {
    expect(buildHighlightDraft("one   two", 3, 6)).toBeNull();
  });
});

describe("placeHighlights", () => {
  it("uses context to pick the right repeated phrase", () => {
    const text = "Rest well. Then work. Later, rest well. Then sleep.";
    const placed = placeHighlights(text, [
      highlight("h1", "rest well", "Later, ", ". Then sleep."),
    ]);
    expect(placed).toEqual([
      {
        id: "h1",
        start: text.lastIndexOf("rest well"),
        end: text.lastIndexOf("rest well") + 9,
      },
    ]);
  });

  it("matches across different whitespace", () => {
    const text = "First paragraph ends.\n\nSecond   paragraph starts.";
    const [placed] = placeHighlights(text, [
      highlight("h1", "ends. Second paragraph"),
    ]);
    expect(text.slice(placed.start, placed.end)).toBe(
      "ends.\n\nSecond   paragraph",
    );
  });

  it("skips highlights that are gone or overlap an earlier one", () => {
    const text = "One two three four.";
    const placed = placeHighlights(text, [
      highlight("a", "two three"),
      highlight("b", "three four"),
      highlight("c", "not here"),
      highlight("d", "One"),
    ]);
    expect(placed.map((p) => p.id)).toEqual(["d", "a"]);
  });
});

function group(
  channel: string,
  videos: { id: string; title: string; day: string; texts: string[] }[],
): HighlightChannelGroup {
  return {
    source_id: channel,
    channel_id: channel,
    provider: "you_tube",
    source_kind: "you_tube_channel",
    channel_name: `Channel ${channel}`,
    channel_thumbnail_url: null,
    videos: videos.map((video) => ({
      source_id: channel,
      video_id: video.id,
      item_id: video.id,
      provider: "you_tube",
      item_kind: "video",
      title: video.title,
      thumbnail_url: null,
      published_at: `2026-10-${video.day}T08:00:00Z`,
      highlights: video.texts.map((text, index) =>
        highlight(`${video.id}-${index}`, text),
      ),
    })),
  };
}

describe("highlight collection", () => {
  const stories = listHighlightedStories([
    group("a", [
      {
        id: "v1",
        title: "On sleep",
        day: "01",
        texts: ["Adenosine builds up", "Naps help"],
      },
      { id: "v2", title: "On work", day: "06", texts: ["Small teams ship"] },
    ]),
    group("b", [
      {
        id: "v3",
        title: "Databases",
        day: "03",
        texts: ["Postgres was enough"],
      },
    ]),
  ]);

  it("lists stories newest first, across channels", () => {
    expect(stories.map((s) => s.videoId)).toEqual(["v2", "v3", "v1"]);
    expect(stories[1].channelName).toBe("Channel b");
  });

  it("counts every highlight", () => {
    expect(countHighlights(stories)).toBe(4);
  });

  it("filters by highlight text, title, and channel, dropping empty stories", () => {
    const bySleep = filterHighlightedStories(stories, "sleep");
    expect(countHighlights(bySleep)).toBe(2);
    expect(bySleep.map((s) => s.videoId)).toEqual(["v1"]);

    const byWords = filterHighlightedStories(stories, "channel b postgres");
    expect(countHighlights(byWords)).toBe(1);

    expect(filterHighlightedStories(stories, "  ")).toBe(stories);
  });

  it("removes one highlight and any story left empty", () => {
    const left = withoutHighlight(stories, "v3-0");
    expect(left.map((s) => s.videoId)).toEqual(["v2", "v1"]);
    expect(countHighlights(left)).toBe(3);
  });
});
