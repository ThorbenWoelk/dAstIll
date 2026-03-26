import { describe, expect, it } from "bun:test";

import type { Video } from "../src/lib/types";
import {
  buildOptimisticAcknowledgeSidebarList,
  isStillSelectedAfterAcknowledgeSuccess,
  matchesAcknowledgedFilterVideo,
  resolveRevertedVideoForAcknowledge,
  resolveVideoForAcknowledgeToggle,
  selectionDroppedAfterAcknowledgeOptimistic,
} from "../src/lib/workspace/acknowledge-toggle";

function makeVideo(
  overrides: Partial<Video> & { id: string; acknowledged: boolean },
): Video {
  return {
    channel_id: "ch-1",
    title: "t",
    published_at: new Date().toISOString(),
    transcript_status: "ready",
    summary_status: "ready",
    ...overrides,
  } as Video;
}

describe("resolveVideoForAcknowledgeToggle", () => {
  it("returns list row when the id is in sidebar videos", () => {
    const v = makeVideo({ id: "v-1", acknowledged: false });
    const r = resolveVideoForAcknowledgeToggle([v], "v-1", null);
    expect(r).toEqual({ video: v, videoFromList: true });
  });

  it("returns pending-only row when id is not in the loaded page", () => {
    const pending = makeVideo({ id: "v-deep", acknowledged: false });
    const r = resolveVideoForAcknowledgeToggle([], "v-deep", pending);
    expect(r).toEqual({ video: pending, videoFromList: false });
  });

  it("returns null when id is missing from both list and pending", () => {
    const v = makeVideo({ id: "v-1", acknowledged: false });
    expect(resolveVideoForAcknowledgeToggle([v], "v-other", null)).toBeNull();
    expect(
      resolveVideoForAcknowledgeToggle(
        [v],
        "v-1",
        makeVideo({ id: "v-x", acknowledged: false }),
      ),
    ).toEqual({ video: v, videoFromList: true });
  });

  it("returns null when selectedVideoId is null", () => {
    expect(resolveVideoForAcknowledgeToggle([], null, null)).toBeNull();
  });
});

describe("matchesAcknowledgedFilterVideo", () => {
  const vRead = makeVideo({ id: "a", acknowledged: true });
  const vUnread = makeVideo({ id: "b", acknowledged: false });

  it("filters ack / unack / all", () => {
    expect(matchesAcknowledgedFilterVideo(vRead, "ack")).toBe(true);
    expect(matchesAcknowledgedFilterVideo(vUnread, "ack")).toBe(false);
    expect(matchesAcknowledgedFilterVideo(vUnread, "unack")).toBe(true);
    expect(matchesAcknowledgedFilterVideo(vRead, "unack")).toBe(false);
    expect(matchesAcknowledgedFilterVideo(vRead, "all")).toBe(true);
  });
});

describe("buildOptimisticAcknowledgeSidebarList", () => {
  it("returns previous snapshot when selection is pending-only", () => {
    const prev = [makeVideo({ id: "a", acknowledged: false })];
    const out = buildOptimisticAcknowledgeSidebarList(
      false,
      prev,
      [],
      "missing",
      true,
      "all",
    );
    expect(out).toBe(prev);
  });

  it("applies acknowledge and filter when row is in list", () => {
    const v = makeVideo({ id: "v1", acknowledged: false });
    const out = buildOptimisticAcknowledgeSidebarList(
      true,
      [],
      [v],
      "v1",
      true,
      "unack",
    );
    expect(out).toEqual([]);
  });
});

describe("selectionDroppedAfterAcknowledgeOptimistic", () => {
  it("detects drop from list when row disappears under filter", () => {
    const prevId = "v1";
    const opt = makeVideo({ id: "v1", acknowledged: true });
    expect(
      selectionDroppedAfterAcknowledgeOptimistic(
        true,
        [],
        prevId,
        opt,
        "unack",
      ),
    ).toBe(true);
  });

  it("detects drop for pending-only when marked read under unack filter", () => {
    const opt = makeVideo({ id: "v1", acknowledged: true });
    expect(
      selectionDroppedAfterAcknowledgeOptimistic(false, [], "v1", opt, "unack"),
    ).toBe(true);
  });

  it("keeps selection when filter still includes row", () => {
    const list = [makeVideo({ id: "v1", acknowledged: true })];
    expect(
      selectionDroppedAfterAcknowledgeOptimistic(
        true,
        list,
        "v1",
        list[0],
        "all",
      ),
    ).toBe(false);
  });
});

describe("isStillSelectedAfterAcknowledgeSuccess", () => {
  it("is true when id is in list", () => {
    const v = makeVideo({ id: "v1", acknowledged: true });
    expect(isStillSelectedAfterAcknowledgeSuccess("v1", [v], null)).toBe(true);
  });

  it("is true when id matches pending only", () => {
    const p = makeVideo({ id: "v1", acknowledged: true });
    expect(isStillSelectedAfterAcknowledgeSuccess("v1", [], p)).toBe(true);
  });

  it("is false when nothing matches selected id", () => {
    const v = makeVideo({ id: "v1", acknowledged: true });
    expect(isStillSelectedAfterAcknowledgeSuccess("v2", [v], null)).toBe(false);
  });
});

describe("resolveRevertedVideoForAcknowledge", () => {
  it("prefers list then pending", () => {
    const listV = makeVideo({ id: "x", acknowledged: false });
    const pend = makeVideo({ id: "x", acknowledged: true });
    expect(resolveRevertedVideoForAcknowledge([listV], "x", pend)).toBe(listV);
    expect(resolveRevertedVideoForAcknowledge([], "x", pend)).toBe(pend);
    expect(resolveRevertedVideoForAcknowledge([], "y", pend)).toBeNull();
  });
});
