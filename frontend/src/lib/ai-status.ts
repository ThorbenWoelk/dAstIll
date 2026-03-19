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
        detail: "AI actions will use cloud models.",
        title: "Cloud models available",
      };
    case "local_only":
      return {
        dotClass: "bg-[var(--status-warn)]",
        detail:
          "Cloud models are unavailable right now. AI actions will use local fallback models.",
        title: "Local models only",
      };
    case "offline":
      return {
        dotClass: "bg-[var(--status-error)]",
        detail:
          "This showcase deployment has AI turned off. Everything else remains available.",
        title: "Showcase mode",
      };
  }
}
