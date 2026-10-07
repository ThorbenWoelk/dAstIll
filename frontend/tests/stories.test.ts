import { describe, expect, it } from "bun:test";

import {
  chooseLeadStory,
  collectUnreadStories,
  describeReleaseDay,
  describeStoriesLeft,
  FRONT_PAGE,
  insertStory,
  listSections,
  readingMinutes,
  storiesInSection,
} from "../src/lib/edition/stories";
import { channel, summaryItem } from "./fixtures";

describe("collectUnreadStories", () => {
  it("orders every channel's stories by release date, newest first", () => {
    const stories = collectUnreadStories([
      summaryItem("old", {
        channel_id: "c1",
        published_at: "2026-10-01T08:00:00Z",
      }),
      summaryItem("new", {
        channel_id: "c2",
        published_at: "2026-10-05T08:00:00Z",
      }),
      summaryItem("mid", {
        channel_id: "c1",
        published_at: "2026-10-03T08:00:00Z",
      }),
    ]);
    expect(stories.map((s) => s.id)).toEqual(["new", "mid", "old"]);
  });

  it("drops read items, empty summaries and duplicates", () => {
    const stories = collectUnreadStories([
      summaryItem("a"),
      summaryItem("a"),
      summaryItem("read", { read: true }),
      summaryItem("empty", { summary_content: "   " }),
    ]);
    expect(stories.map((s) => s.id)).toEqual(["a"]);
  });

  it("puts stories without a release date last", () => {
    const stories = collectUnreadStories([
      summaryItem("undated", { published_at: null }),
      summaryItem("dated"),
    ]);
    expect(stories.map((s) => s.id)).toEqual(["dated", "undated"]);
  });
});

describe("sections", () => {
  const stories = collectUnreadStories([
    summaryItem("a1", {
      channel_id: "a",
      published_at: "2026-10-04T00:00:00Z",
    }),
    summaryItem("b1", {
      channel_id: "b",
      published_at: "2026-10-03T00:00:00Z",
    }),
    summaryItem("a2", {
      channel_id: "a",
      published_at: "2026-10-02T00:00:00Z",
    }),
  ]);

  it("shows everything on the front page and one channel per section", () => {
    expect(storiesInSection(stories, FRONT_PAGE)).toHaveLength(3);
    expect(storiesInSection(stories, "a").map((s) => s.id)).toEqual([
      "a1",
      "a2",
    ]);
  });

  it("lists the front page first, then channels by name with unread counts", () => {
    const sections = listSections(
      [channel("b", "Zeta"), channel("a", "Alpha"), channel("c", "Mu")],
      stories,
    );
    expect(sections).toEqual([
      { id: FRONT_PAGE, name: "Front page", unread: 3 },
      { id: "a", name: "Alpha", unread: 2 },
      { id: "c", name: "Mu", unread: 0 },
      { id: "b", name: "Zeta", unread: 1 },
    ]);
  });
});

describe("chooseLeadStory", () => {
  const stories = collectUnreadStories([
    summaryItem("newest", { published_at: "2026-10-05T00:00:00Z" }),
    summaryItem("older", { published_at: "2026-10-01T00:00:00Z" }),
  ]);

  it("leads with the newest story", () => {
    expect(chooseLeadStory(stories, null)?.id).toBe("newest");
  });

  it("leads with a picked story while it is still visible", () => {
    expect(chooseLeadStory(stories, "older")?.id).toBe("older");
    expect(chooseLeadStory(stories, "gone")?.id).toBe("newest");
  });

  it("has no lead when the section is empty", () => {
    expect(chooseLeadStory([], null)).toBeNull();
  });
});

describe("insertStory", () => {
  it("puts an undone story back in release order without duplicates", () => {
    const [newest, middle, oldest] = collectUnreadStories([
      summaryItem("n", { published_at: "2026-10-05T00:00:00Z" }),
      summaryItem("m", { published_at: "2026-10-03T00:00:00Z" }),
      summaryItem("o", { published_at: "2026-10-01T00:00:00Z" }),
    ]);
    const restored = insertStory(insertStory([newest, oldest], middle), middle);
    expect(restored.map((s) => s.id)).toEqual(["n", "m", "o"]);
  });
});

describe("describeReleaseDay", () => {
  const now = new Date(2026, 9, 6, 12, 0); // Tuesday 6 October 2026

  it("uses words for recent days", () => {
    expect(describeReleaseDay(new Date(2026, 9, 6, 7).toISOString(), now)).toBe(
      "Today",
    );
    expect(
      describeReleaseDay(new Date(2026, 9, 5, 23).toISOString(), now),
    ).toBe("Yesterday");
    expect(describeReleaseDay(new Date(2026, 9, 3, 9).toISOString(), now)).toBe(
      "Saturday",
    );
  });

  it("falls back to a date after a week, with the year only when it differs", () => {
    expect(describeReleaseDay(new Date(2026, 8, 20).toISOString(), now)).toBe(
      "20 Sept",
    );
    expect(describeReleaseDay(new Date(2025, 11, 24).toISOString(), now)).toBe(
      "24 Dec 2025",
    );
  });

  it("returns nothing for missing or broken dates", () => {
    expect(describeReleaseDay(null, now)).toBe("");
    expect(describeReleaseDay("not a date", now)).toBe("");
  });
});

describe("small labels", () => {
  it("estimates reading time in whole minutes, at least one", () => {
    expect(readingMinutes("short")).toBe(1);
    expect(readingMinutes(Array(660).fill("word").join(" "))).toBe(3);
  });

  it("counts a hyphenated word once and skips list dashes", () => {
    // 330 list items of "well-known" are 330 words: 1.5 minutes rounds to 2.
    const list = Array(330).fill("- well-known").join("\n");
    expect(readingMinutes(list)).toBe(2);
    // Counted as two words each, the same text would read as 3 minutes.
    expect(readingMinutes(Array(660).fill("word").join(" "))).toBe(3);
  });

  it("counts stories left", () => {
    expect(describeStoriesLeft(0)).toBe("No stories left");
    expect(describeStoriesLeft(1)).toBe("1 story left");
    expect(describeStoriesLeft(7)).toBe("7 stories left");
  });
});
