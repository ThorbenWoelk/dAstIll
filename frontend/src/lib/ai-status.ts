import type { AiStatus } from "./types";

export type { AiStatus };

export function resolveAiIndicatorPresentation(status: AiStatus): {
  dotClass: string;
  detail: string;
  title: string;
} {
  switch (status) {
    case "cloud":
      return {
        dotClass: "bg-green-500",
        detail:
          "Primary cloud models are reachable. AI actions will use the cloud path.",
        title: "Cloud models available",
      };
    case "local_only":
      return {
        dotClass: "bg-slate-400",
        detail:
          "Cloud models are currently unavailable or cooling down. AI actions will use local fallback models only.",
        title: "Local models only",
      };
    case "offline":
      return {
        dotClass: "bg-red-500",
        detail:
          "This is a showcase deployment - AI features are turned off. Browsing channels, inspecting summaries, and all other non-AI features are fully functional.",
        title: "Showcase mode",
      };
  }
}
