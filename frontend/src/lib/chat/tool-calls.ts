import type { ChatStreamStatus, ChatToolCall } from "$lib/types";

export function deriveToolCalls(statuses: ChatStreamStatus[]): ChatToolCall[] {
  const merged = new Map<string, ChatToolCall>();

  for (const status of statuses) {
    const tool = status.tool;
    if (!tool) continue;

    const key = `${tool.name}:${tool.input}`;
    const existing = merged.get(key);
    merged.set(key, {
      name: tool.name,
      label: tool.label,
      state: tool.state,
      input: tool.input,
      output: tool.output ?? existing?.output ?? null,
    });
  }

  return [...merged.values()];
}
