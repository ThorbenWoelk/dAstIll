import type { ChatMessage } from "$lib/types";

function coerceNumber(value: unknown): number | undefined {
  if (value == null) return undefined;
  if (typeof value === "bigint") return Number(value);
  if (typeof value === "number" && Number.isFinite(value)) return value;
  return undefined;
}

function formatTokenCount(value: number): string {
  if (value >= 10_000) {
    return `${Math.round(value / 1000)}k`;
  }
  if (value >= 1000) {
    const k = value / 1000;
    const rounded = Math.round(k * 10) / 10;
    return rounded % 1 === 0 ? `${rounded}k` : `${rounded}k`;
  }
  return String(value);
}

function formatDurationNs(ns: number): string | null {
  if (!Number.isFinite(ns) || ns <= 0) {
    return null;
  }
  const sec = ns / 1e9;
  if (sec < 60) {
    return sec < 10 ? `${sec.toFixed(1)}s` : `${Math.round(sec)}s`;
  }
  const minutes = Math.floor(sec / 60);
  const rest = Math.round(sec - minutes * 60);
  return `${minutes}m ${rest}s`;
}

/** One-line metadata under an assistant message (model, token counts, Ollama-reported duration). */
export function formatAssistantResponseStats(
  message: ChatMessage,
  options: { loading: boolean },
): string | null {
  if (message.role !== "assistant" || options.loading) {
    return null;
  }
  if (message.status !== "completed") {
    return null;
  }

  const model = message.model?.trim();
  const promptTokens = coerceNumber(message.prompt_tokens);
  const completionTokens = coerceNumber(message.completion_tokens);
  const durationNs = coerceNumber(message.total_duration_ns);

  if (
    !model &&
    promptTokens == null &&
    completionTokens == null &&
    (durationNs == null || durationNs <= 0)
  ) {
    return null;
  }

  const parts: string[] = [];
  if (model) {
    parts.push(model);
  }
  if (promptTokens != null) {
    parts.push(`${formatTokenCount(promptTokens)} in`);
  }
  if (completionTokens != null) {
    parts.push(`${formatTokenCount(completionTokens)} out`);
  }
  const duration = durationNs != null ? formatDurationNs(durationNs) : null;
  if (duration) {
    parts.push(duration);
  }

  return parts.join(" · ");
}
