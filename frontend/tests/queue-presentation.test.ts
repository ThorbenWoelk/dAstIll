import { describe, expect, it } from "bun:test";

import type { Video } from "../src/lib/types";
import {
  queueStateAccentClass,
  queueVideoPipelineSteps,
  queueVideoPrimaryState,
} from "../src/lib/queue/presentation";

function makeVideo(
  overrides: Partial<Video> & {
    id: string;
    transcript_status: Video["transcript_status"];
    summary_status: Video["summary_status"];
  },
): Video {
  return {
    id: overrides.id,
    channel_id: overrides.channel_id ?? "channel-1",
    title: overrides.title ?? "Video",
    thumbnail_url: null,
    published_at: "2024-01-01T00:00:00Z",
    is_short: false,
    transcript_status: overrides.transcript_status,
    summary_status: overrides.summary_status,
    acknowledged: false,
    ...overrides,
  };
}

describe("queueVideoPrimaryState", () => {
  it("prioritizes transcript failures", () => {
    expect(
      queueVideoPrimaryState(
        makeVideo({
          id: "video-1",
          transcript_status: "failed",
          summary_status: "pending",
        }),
      ),
    ).toBe("Transcript failed");
  });

  it("reports summary generation after transcript readiness", () => {
    expect(
      queueVideoPrimaryState(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "loading",
        }),
      ),
    ).toBe("Summary generating");
  });

  it("reports complete when both stages are ready", () => {
    expect(
      queueVideoPrimaryState(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "ready",
        }),
      ),
    ).toBe("Complete");
  });
});

describe("queueVideoPipelineSteps", () => {
  it("marks transcript loading as the active step", () => {
    expect(
      queueVideoPipelineSteps(
        makeVideo({
          id: "video-1",
          transcript_status: "loading",
          summary_status: "pending",
        }),
      ),
    ).toEqual([
      { key: "q", label: "Queue", status: "complete" },
      { key: "tr", label: "Transcript", status: "active" },
      { key: "su", label: "Summary", status: "upcoming" },
    ]);
  });

  it("marks summary failure after transcript completion", () => {
    expect(
      queueVideoPipelineSteps(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "failed",
        }),
      ),
    ).toEqual([
      { key: "q", label: "Queue", status: "complete" },
      { key: "tr", label: "Transcript", status: "complete" },
      { key: "su", label: "Summary", status: "failed" },
    ]);
  });
});

describe("queueStateAccentClass", () => {
  it("uses the danger accent for failures", () => {
    expect(
      queueStateAccentClass(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "failed",
        }),
      ),
    ).toBe("bg-[var(--danger)]");
  });

  it("uses the active accent for loading states", () => {
    expect(
      queueStateAccentClass(
        makeVideo({
          id: "video-1",
          transcript_status: "loading",
          summary_status: "pending",
        }),
      ),
    ).toBe("bg-[var(--accent)] motion-safe:animate-pulse");
  });

  it("uses the settled accent for completed work", () => {
    expect(
      queueStateAccentClass(
        makeVideo({
          id: "video-1",
          transcript_status: "ready",
          summary_status: "ready",
        }),
      ),
    ).toBe("bg-[var(--accent-strong)]/80");
  });
});
