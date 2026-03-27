import { describe, expect, it } from "bun:test";

import { deriveToolCalls } from "../src/lib/chat/tool-calls";
import type { ChatStreamStatus } from "../src/lib/types";

describe("deriveToolCalls", () => {
  it("merges running and completed events for the same tool call", () => {
    const statuses: ChatStreamStatus[] = [
      {
        stage: "tool",
        tool: {
          name: "db_inspect",
          label: "Database inspect",
          state: "running",
          input: "Count summaries in the database",
        },
      },
      {
        stage: "tool_complete",
        tool: {
          name: "db_inspect",
          label: "Database inspect",
          state: "completed",
          input: "Count summaries in the database",
          output: "There are 17 summaries in the database.",
        },
      },
    ];

    expect(deriveToolCalls(statuses)).toEqual([
      {
        name: "db_inspect",
        label: "Database inspect",
        state: "completed",
        input: "Count summaries in the database",
        output: "There are 17 summaries in the database.",
      },
    ]);
  });

  it("keeps distinct tool calls separate", () => {
    const statuses: ChatStreamStatus[] = [
      {
        stage: "tool_complete",
        tool: {
          name: "db_inspect",
          label: "Database inspect",
          state: "completed",
          input: "Count summaries in the database",
          output: "There are 17 summaries in the database.",
        },
      },
      {
        stage: "tool_complete",
        tool: {
          name: "db_inspect",
          label: "Database inspect",
          state: "completed",
          input: "List up to 3 videos from the database",
          output: "Here are the first 3 videos in the database:\n- a - One",
        },
      },
    ];

    expect(deriveToolCalls(statuses)).toHaveLength(2);
  });
});
