import { browser } from "$app/environment";

import { isAiAvailable } from "$lib/api";
import type { AiStatus } from "$lib/types";

type AiStatusPollerOptions = {
  intervalMs?: number;
  onStatus: (payload: { available: boolean; status: AiStatus }) => void;
};

export async function refreshAiStatus(
  onStatus: AiStatusPollerOptions["onStatus"],
) {
  const status = await isAiAvailable();
  onStatus(status);
  return status;
}

export function createAiStatusPoller({
  intervalMs = 30000,
  onStatus,
}: AiStatusPollerOptions) {
  if (!browser) {
    return () => {};
  }

  void refreshAiStatus(onStatus).catch(() => {});

  const timer = window.setInterval(() => {
    void refreshAiStatus(onStatus).catch(() => {});
  }, intervalMs);

  return () => window.clearInterval(timer);
}
