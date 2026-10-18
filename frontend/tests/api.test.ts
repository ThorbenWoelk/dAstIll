import { afterEach, describe, expect, it } from "bun:test";

import {
  ApiError,
  fetchChannelSummaries,
  setStoryRead,
  useReaderIdentity,
  useTokenSource,
} from "../src/lib/api";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  useTokenSource(async () => null);
  useReaderIdentity(() => null);
});

function stubFetch(response: Response) {
  const calls: { url: string; init: RequestInit }[] = [];
  globalThis.fetch = (async (url: string, init: RequestInit) => {
    calls.push({ url, init });
    return response;
  }) as typeof fetch;
  return calls;
}

describe("api", () => {
  it("sends the ID token and asks for one channel", async () => {
    useTokenSource(async () => "token-123");
    const calls = stubFetch(
      Response.json({ channels: [], selected_channel_id: null, summaries: [] }),
    );

    await fetchChannelSummaries("UC a/b");

    expect(calls[0].url).toBe("/api/mini?channel_id=UC%20a%2Fb");
    const headers = new Headers(calls[0].init.headers);
    expect(headers.get("Authorization")).toBe("Bearer token-123");
    expect(calls[0].init.cache).toBe("no-store");
  });

  it("marks a story read with a JSON body", async () => {
    const calls = stubFetch(Response.json({ video_id: "v1", read: true }));
    await setStoryRead("v1", true);
    expect(calls[0].url).toBe("/api/mini/videos/v1/read");
    expect(calls[0].init.method).toBe("PUT");
    expect(calls[0].init.body).toBe(JSON.stringify({ read: true }));
    expect(new Headers(calls[0].init.headers).get("Content-Type")).toBe(
      "application/json",
    );
  });

  it("turns failures into readable messages", async () => {
    stubFetch(new Response("Sign-in required", { status: 403 }));
    await expect(fetchChannelSummaries()).rejects.toThrow(
      "Please sign in again.",
    );

    stubFetch(new Response("Channel not found", { status: 404 }));
    const error = await fetchChannelSummaries().catch((e) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error.message).toBe("Channel not found");
    expect(error.status).toBe(404);
  });

  it("does not send a request after the reader changes", async () => {
    let readerId: string | null = "user-a";
    useReaderIdentity(() => readerId);
    useTokenSource(async () => {
      readerId = "user-b";
      return "token-a";
    });
    const calls = stubFetch(Response.json({ video_id: "v1", read: true }));

    await expect(setStoryRead("v1", true)).rejects.toThrow(
      "Please sign in again.",
    );
    expect(calls).toHaveLength(0);
  });

  it("reports network failures as connection problems", async () => {
    globalThis.fetch = (async () => {
      throw new TypeError("Failed to fetch");
    }) as unknown as typeof fetch;
    await expect(setStoryRead("v1", true)).rejects.toThrow(
      "Could not reach the server",
    );
  });
});
