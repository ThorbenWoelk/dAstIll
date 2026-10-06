import { describe, expect, it } from "bun:test";

import { trimToStorageBudget } from "../src/lib/edition/keepsake";
import { collectUnreadStories } from "../src/lib/edition/stories";
import { channel, summaryItem } from "./fixtures";

const stories = collectUnreadStories(
  Array.from({ length: 40 }, (_, i) =>
    summaryItem(`v${String(i).padStart(2, "0")}`, {
      published_at: new Date(Date.UTC(2026, 9, 1, 0, i)).toISOString(),
      summary_content: "x".repeat(1000),
    }),
  ),
);

describe("trimToStorageBudget", () => {
  it("keeps everything when it fits", () => {
    const record = trimToStorageBudget([channel("c1")], stories, 1_000_000);
    expect(record.stories).toHaveLength(40);
  });

  it("drops the oldest stories until the edition fits", () => {
    const record = trimToStorageBudget([channel("c1")], stories, 12_000);
    expect(JSON.stringify(record).length).toBeLessThanOrEqual(12_000);
    expect(record.stories.length).toBeGreaterThan(0);
    expect(record.stories.length).toBeLessThan(40);
    expect(record.stories[0].id).toBe(stories[0].id);
  });
});
