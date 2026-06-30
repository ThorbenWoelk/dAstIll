import { afterEach, describe, expect, it } from "bun:test";

import {
  deleteAllConversations,
  flushPendingStreamEvent,
} from "../src/lib/chat/requests";

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

describe("flushPendingStreamEvent", () => {
  it("resets empty pending events without dispatching handlers", () => {
    const pendingEvent = { eventName: "status", dataLines: [] as string[] };
    let called = false;

    flushPendingStreamEvent(pendingEvent, {
      onStatus: () => {
        called = true;
      },
    });

    expect(called).toBeFalse();
    expect(pendingEvent).toEqual({ eventName: "message", dataLines: [] });
  });

  it("dispatches buffered status payloads and resets the pending event", () => {
    const pendingEvent = {
      eventName: "status",
      dataLines: [
        JSON.stringify({
          stage: "generating",
          label: "Answering",
          detail: "Using retrieved evidence",
        }),
      ],
    };
    let receivedStatus: unknown = null;

    flushPendingStreamEvent(pendingEvent, {
      onStatus: (status) => {
        receivedStatus = status;
      },
    });

    expect(receivedStatus).toEqual({
      stage: "generating",
      label: "Answering",
      detail: "Using retrieved evidence",
      decision: null,
      plan: null,
      tool: null,
    });
    expect(pendingEvent).toEqual({ eventName: "message", dataLines: [] });
  });

  it("normalizes technical error payloads before surfacing them", () => {
    const pendingEvent = {
      eventName: "error",
      dataLines: [
        JSON.stringify({ message: "Text-to-Speech is not configured" }),
      ],
    };
    let receivedMessage: string | null = null;

    flushPendingStreamEvent(pendingEvent, {
      onError: (message) => {
        receivedMessage = message;
      },
    });

    expect(receivedMessage).toBe(
      "Sorry, audio playback is not available right now.",
    );
    expect(pendingEvent).toEqual({ eventName: "message", dataLines: [] });
  });
});
