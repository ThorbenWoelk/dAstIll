import type { AiStatus } from "./types";

export type { AiStatus };

export interface AiIndicatorPresentation {
  dotClass: string;
  detail: string;
  title: string;
}

export function resolveAiIndicatorPresentation(
  status: AiStatus,
): AiIndicatorPresentation {
  switch (status) {
    case "cloud":
      return {
        dotClass: "bg-[var(--status-ok)]",
        detail:
          "Primary cloud models are reachable. AI actions will use the cloud path.",
        title: "Cloud models available",
      };
    case "local_only":
      return {
        dotClass: "bg-[var(--status-warn)]",
        detail:
          "Cloud models are currently unavailable or cooling down. AI actions will use local fallback models only.",
        title: "Local models only",
      };
    case "offline":
      return {
        dotClass: "bg-[var(--status-error)]",
        detail:
          "This is a showcase deployment - AI features are turned off. Browsing channels, inspecting summaries, and all other non-AI features are fully functional.",
        title: "Showcase mode",
      };
  }
}
