import { describe, expect, it } from "bun:test";
import {
  resolveAiIndicatorPresentation,
  type AiStatus,
} from "../src/lib/ai-status";

function presentation(status: AiStatus) {
  return resolveAiIndicatorPresentation(status);
}

describe("resolveAiIndicatorPresentation", () => {
  it("renders cloud availability as green ready state", () => {
    expect(presentation("cloud")).toEqual({
      dotClass: "bg-green-500",
      detail:
        "Primary cloud models are reachable. AI actions will use the cloud path.",
      title: "Cloud models available",
    });
  });

  it("renders local-only fallback as grey degraded state", () => {
    expect(presentation("local_only")).toEqual({
      dotClass: "bg-slate-400",
      detail:
        "Cloud models are currently unavailable or cooling down. AI actions will use local fallback models only.",
      title: "Local models only",
    });
  });

  it("renders offline availability as red unavailable state", () => {
    expect(presentation("offline")).toEqual({
      dotClass: "bg-red-500",
      detail:
        "This is a showcase deployment - AI features are turned off. Browsing channels, inspecting summaries, and all other non-AI features are fully functional.",
      title: "Showcase mode",
    });
  });
});
