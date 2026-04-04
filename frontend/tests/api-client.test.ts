import { afterEach, beforeEach, describe, expect, it, mock } from "bun:test";

import { createApiRequestInit, request } from "../src/lib/api-client";
import { configureAuthTokenResolver } from "../src/lib/auth-token";

const originalFetch = globalThis.fetch;

beforeEach(() => {
  configureAuthTokenResolver(async () => "firebase-token-123");
});

afterEach(() => {
  globalThis.fetch = originalFetch;
  configureAuthTokenResolver(async () => null);
});

describe("api client", () => {
  it("adds the Firebase bearer token to JSON requests", async () => {
    const fetchMock = mock(
      async (_input: string | URL | Request, _init?: RequestInit) =>
        new Response(JSON.stringify({ ok: true }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
    );
    globalThis.fetch = fetchMock as typeof fetch;

    await request<{ ok: boolean }>("/api/test");

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [, init] = fetchMock.mock.calls[0]!;
    const headers = new Headers(init?.headers);
    expect(headers.get("Authorization")).toBe("Bearer firebase-token-123");
    expect(headers.get("Content-Type")).toBe("application/json");
  });

  it("can prepare request init without forcing a JSON content type", async () => {
    const init = await createApiRequestInit(
      {
        method: "GET",
      },
      {
        includeJsonContentType: false,
      },
    );

    const headers = new Headers(init.headers);
    expect(headers.get("Authorization")).toBe("Bearer firebase-token-123");
    expect(headers.has("Content-Type")).toBe(false);
  });
});
