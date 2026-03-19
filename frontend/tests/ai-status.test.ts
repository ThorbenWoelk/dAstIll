import { describe, expect, it } from "bun:test";
import {
  resolveAiIndicatorPresentation,
  type AiStatus,
} from "../src/lib/ai-status";

function presentation(status: AiStatus) {
  return resolveAiIndicatorPresentation(status);
}

describe("resolveAiIndicatorPresentation", () => {
  it("renders cloud availability as the ready state", () => {
    expect(presentation("cloud")).toEqual({
      dotClass: "bg-[var(--status-ok)]",
      detail: "AI actions will use cloud models.",
      title: "Cloud models available",
    });
  });

  it("renders local-only fallback as the degraded state", () => {
    expect(presentation("local_only")).toEqual({
      dotClass: "bg-[var(--status-warn)]",
      detail:
        "Cloud models are unavailable right now. AI actions will use local fallback models.",
      title: "Local models only",
    });
  });

  it("renders offline availability as the unavailable state", () => {
    expect(presentation("offline")).toEqual({
      dotClass: "bg-[var(--status-error)]",
      detail:
        "This showcase deployment has AI turned off. Everything else remains available.",
      title: "Showcase mode",
    });
  });
});
