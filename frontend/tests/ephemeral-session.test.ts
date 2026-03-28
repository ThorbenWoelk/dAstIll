import { describe, expect, it } from "bun:test";

import {
  conversationToSummary,
  createEmptyEphemeralConversation,
} from "../src/lib/chat/ephemeral-session";

describe("ephemeral-session", () => {
  it("creates a conversation with stable summary shape", () => {
    const conv = createEmptyEphemeralConversation();
    expect(conv.messages).toEqual([]);
    expect(conv.id.startsWith("conv_")).toBe(true);
    const summary = conversationToSummary(conv);
    expect(summary.id).toBe(conv.id);
    expect(summary.title_status).toBe(conv.title_status);
  });
});
