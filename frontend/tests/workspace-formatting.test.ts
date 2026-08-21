import { describe, expect, it } from "bun:test";
import {
  bumpTranscriptRevision,
  currentTranscriptRevision,
  shouldPersistBackgroundTranscriptWrite,
} from "../src/lib/workspace/formatting";

describe("transcript revision", () => {
  it("starts at zero and bumps per video", () => {
    const revisions: Record<string, number> = {};
    expect(currentTranscriptRevision(revisions, "video-a")).toBe(0);
    expect(bumpTranscriptRevision(revisions, "video-a")).toBe(1);
    expect(bumpTranscriptRevision(revisions, "video-a")).toBe(2);
    expect(currentTranscriptRevision(revisions, "video-b")).toBe(0);
  });
});

describe("shouldPersistBackgroundTranscriptWrite", () => {
  it("persists formatted output when the transcript was not edited during the job", () => {
    expect(
      shouldPersistBackgroundTranscriptWrite({
        resultDiffersFromSource: true,
        capturedRevision: 0,
        currentRevision: 0,
      }),
    ).toBe(true);
  });

  it("does not persist when a later save, reset, or revert bumped the revision", () => {
    const revisions: Record<string, number> = {};
    const capturedRevision = currentTranscriptRevision(revisions, "video-a");
    bumpTranscriptRevision(revisions, "video-a");
    expect(
      shouldPersistBackgroundTranscriptWrite({
        resultDiffersFromSource: true,
        capturedRevision,
        currentRevision: currentTranscriptRevision(revisions, "video-a"),
      }),
    ).toBe(false);
  });

  it("does not persist when formatting produced no changes", () => {
    expect(
      shouldPersistBackgroundTranscriptWrite({
        resultDiffersFromSource: false,
        capturedRevision: 0,
        currentRevision: 0,
      }),
    ).toBe(false);
  });

  it("does not treat another video's save as a mutation of this video", () => {
    const revisions: Record<string, number> = {};
    const capturedRevision = currentTranscriptRevision(revisions, "video-a");
    bumpTranscriptRevision(revisions, "video-b");
    expect(
      shouldPersistBackgroundTranscriptWrite({
        resultDiffersFromSource: true,
        capturedRevision,
        currentRevision: currentTranscriptRevision(revisions, "video-a"),
      }),
    ).toBe(true);
  });
});
