import { describe, expect, it } from "bun:test";

import {
  generateSummaryAudio,
  readSummaryAudioSession,
  resetSummaryAudioSessionsForTesting,
  resolveSummaryAudioTimelineState,
  syncSummaryAudioDebugState,
} from "../src/lib/workspace/summary-audio-session";

function deferredResponse() {
  let resolve!: (response: Response) => void;
  const promise = new Promise<Response>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

describe("summary audio session", () => {
  it("keeps generating state while the generation request is still in flight", async () => {
    resetSummaryAudioSessionsForTesting();
    const request = deferredResponse();

    const generation = generateSummaryAudio("video-1", () => request.promise);
    expect(readSummaryAudioSession("video-1").status).toBe("generating");

    syncSummaryAudioDebugState("video-1", {
      cache_hit: false,
      word_count: 120,
      estimated_secs: 7,
    });

    expect(readSummaryAudioSession("video-1")).toMatchObject({
      status: "generating",
      summaryWordCount: 120,
      estimatedSecs: 7,
    });

    request.resolve(new Response("", { status: 200 }));
    await generation;

    expect(readSummaryAudioSession("video-1")).toMatchObject({
      status: "ready",
      audioSrc: "/api/videos/video-1/summary/audio",
    });
  });

  it("returns a neutral timeline state until audio duration is known", () => {
    resetSummaryAudioSessionsForTesting();

    expect(resolveSummaryAudioTimelineState(132, 0)).toEqual({
      knownDuration: false,
      sliderMax: 100,
      sliderValue: 0,
      progressPercent: 0,
    });
  });

  it("uses real duration once metadata is available", () => {
    resetSummaryAudioSessionsForTesting();

    expect(resolveSummaryAudioTimelineState(45, 180)).toEqual({
      knownDuration: true,
      sliderMax: 180,
      sliderValue: 45,
      progressPercent: 25,
    });
  });
});
