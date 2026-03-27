import { afterEach, describe, expect, it } from "bun:test";

import { deleteAllConversations } from "../src/lib/chat-api";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
});

describe("deleteAllConversations", () => {
  it("issues one delete request to the bulk conversation endpoint", async () => {
    const requests: string[] = [];

    globalThis.fetch = (async (input, init) => {
      requests.push(
        `${(init?.method ?? "GET").toUpperCase()} ${String(input)}`,
      );
      return new Response(null, { status: 204 });
    }) as typeof fetch;

    await expect(deleteAllConversations()).resolves.toBeUndefined();
    expect(requests).toEqual(["DELETE /api/chat/conversations"]);
  });
});
