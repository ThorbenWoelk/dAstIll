import { describe, expect, it } from "bun:test";

import {
  createHomeTourSteps,
  type TourContext,
} from "../src/lib/workspace/home-tour";

function minimalTourContext(overrides: Partial<TourContext> = {}): TourContext {
  return {
    mobileBrowseOpen: false,
    selectedVideoId: null,
    selectedChannelId: null,
    videos: [],
    contentMode: "info",
    isAuthenticated: () => false,
    selectVideo: async () => {},
    setMode: () => {},
    tick: async () => {},
    ...overrides,
  };
}

describe("createHomeTourSteps", () => {
  it("exposes a stable first step title for the workspace feature guide", () => {
    const steps = createHomeTourSteps(minimalTourContext());
    expect(steps.length).toBe(10);
    expect(steps[0]?.title).toBe("Welcome to dAstIll");
  });
});
