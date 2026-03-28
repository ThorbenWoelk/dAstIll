import { afterEach, describe, expect, it, mock } from "bun:test";

mock.module("$app/environment", () => ({
  dev: false,
}));

mock.module("$env/dynamic/private", () => ({
  env: process.env,
}));

const originalFetch = globalThis.fetch;
const originalEnv = {
  BACKEND_API_BASE: process.env.BACKEND_API_BASE,
  BACKEND_PROXY_TOKEN: process.env.BACKEND_PROXY_TOKEN,
  BACKEND_IDENTITY_AUDIENCE: process.env.BACKEND_IDENTITY_AUDIENCE,
};

afterEach(() => {
  globalThis.fetch = originalFetch;
  for (const [key, value] of Object.entries(originalEnv)) {
    if (value === undefined) {
      delete process.env[key];
      continue;
    }

    process.env[key] = value;
  }
});

async function loadRouteModule() {
  return import(
    `../src/routes/api/[...path]/+server.ts?test=${Date.now()}-${Math.random()}`
  );
}

describe("/api/[...path] proxy route", () => {
  it("strips client-supplied x-dastill headers and forwards authenticated identity headers", async () => {
    process.env.BACKEND_API_BASE = "http://backend.example.com";
    process.env.BACKEND_PROXY_TOKEN = "proxy-secret";
    delete process.env.BACKEND_IDENTITY_AUDIENCE;

    const route = await loadRouteModule();
    const backendFetch = mock(
      async (_input: RequestInfo | URL, _init?: RequestInit) =>
        new Response(null, {
          status: 200,
          headers: {
            "content-type": "application/json",
          },
        }),
    );
    globalThis.fetch = backendFetch as typeof fetch;

    await route.GET({
      params: { path: "channels" },
      url: new URL("http://localhost:3543/api/channels?limit=10"),
      request: new Request("http://localhost:3543/api/channels?limit=10", {
        headers: {
          "x-dastill-role": "operator",
          "x-dastill-auth-state": "authenticated",
          "x-dastill-user-id": "spoofed-user",
          "x-dastill-client-ip": "0.0.0.0",
          "x-dastill-proxy-auth": "spoofed-token",
          "x-custom-header": "kept",
        },
      }),
      locals: {
        auth: {
          userId: "firebase-uid-123",
          authState: "authenticated",
          accessRole: "operator",
          email: "operator@example.com",
        },
      },
      getClientAddress: () => "203.0.113.10",
    });

    expect(backendFetch).toHaveBeenCalledTimes(1);
    const [, init] = backendFetch.mock.calls[0]!;
    const headers = new Headers(init?.headers);
    expect(headers.get("x-custom-header")).toBe("kept");
    expect(headers.get("x-dastill-proxy-auth")).toBe("proxy-secret");
    expect(headers.get("x-dastill-role")).toBe("operator");
    expect(headers.get("x-dastill-auth-state")).toBe("authenticated");
    expect(headers.get("x-dastill-user-id")).toBe("firebase-uid-123");
    expect(headers.get("x-dastill-client-ip")).toBe("203.0.113.10");
  });

  it("forwards anonymous requests with an empty user id", async () => {
    process.env.BACKEND_API_BASE = "http://backend.example.com";
    process.env.BACKEND_PROXY_TOKEN = "proxy-secret";
    delete process.env.BACKEND_IDENTITY_AUDIENCE;

    const route = await loadRouteModule();
    const backendFetch = mock(
      async (_input: RequestInfo | URL, _init?: RequestInit) =>
        new Response(null, {
          status: 200,
          headers: {
            "content-type": "application/json",
          },
        }),
    );
    globalThis.fetch = backendFetch as typeof fetch;

    await route.GET({
      params: { path: "channels" },
      url: new URL("http://localhost:3543/api/channels"),
      request: new Request("http://localhost:3543/api/channels", {
        headers: {
          "x-dastill-user-id": "spoofed-user",
        },
      }),
      locals: {
        auth: {
          userId: "anonymous-firebase-uid",
          authState: "anonymous",
          accessRole: "anonymous",
          email: null,
        },
      },
      getClientAddress: () => "198.51.100.24",
    });

    expect(backendFetch).toHaveBeenCalledTimes(1);
    const [, init] = backendFetch.mock.calls[0]!;
    const headers = new Headers(init?.headers);
    expect(headers.get("x-dastill-role")).toBe("anonymous");
    expect(headers.get("x-dastill-auth-state")).toBe("anonymous");
    expect(headers.get("x-dastill-user-id")).toBe("");
    expect(headers.get("x-dastill-client-ip")).toBe("198.51.100.24");
  });
});
