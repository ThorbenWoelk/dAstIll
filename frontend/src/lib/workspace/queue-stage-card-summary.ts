import type { QueueTab, Video } from "$lib/types";

/** One-line label for queue stage counts (channel cards, tooltips). */
export function queueStageCardSummary(
  videos: Video[],
  tab: QueueTab,
): string | null {
  if (videos.length === 0) {
    return "Clear for this stage";
  }
  if (tab === "transcripts") {
    const loading = videos.filter(
      (v) => v.transcript_status === "loading",
    ).length;
    const pending = videos.filter(
      (v) => v.transcript_status === "pending",
    ).length;
    const failed = videos.filter(
      (v) => v.transcript_status === "failed",
    ).length;
    return `${videos.length} in queue · ${pending} waiting · ${loading} active · ${failed} failed`;
  }
  if (tab === "summaries") {
    const loading = videos.filter((v) => v.summary_status === "loading").length;
    const pending = videos.filter((v) => v.summary_status === "pending").length;
    const failed = videos.filter((v) => v.summary_status === "failed").length;
    return `${videos.length} in queue · ${pending} waiting · ${loading} active · ${failed} failed`;
  }
  const unevaluated = videos.filter(
    (v) => v.quality_score === null || v.quality_score === undefined,
  ).length;
  return `${videos.length} in queue · ${unevaluated} need scoring`;
}
