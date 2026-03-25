import type { Summary as SummaryPayload } from "$lib/types";

/** FNV-1a 32-bit hash as 8-char hex (stable summary identity helper). */
export function hashSummarySignature(value: string): string {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

export function deriveSummaryTrackingId(summary: SummaryPayload): string {
  const signature = [
    summary.video_id,
    summary.model_used ?? "",
    summary.content ?? "",
  ].join("\u001f");
  return `${summary.video_id}:${hashSummarySignature(signature)}`;
}
