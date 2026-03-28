import { afterEach, describe, expect, it, mock } from "bun:test";

const mockedServerAuth = {
  AuthSessionError: class AuthSessionError extends Error {
    constructor(
      readonly status: number,
      message: string,
    ) {
      super(message);
    }
  },
  buildAnonymousAuthContext: mock(() => ({
    userId: null,
    authState: "anonymous" as const,
    accessRole: "anonymous" as const,
    email: null,
  })),
  clearAuthSessionCookies: mock(() => undefined),
  createSessionCookieFromIdToken: mock(async () => ({
    auth: {
      userId: "user-123",
      authState: "authenticated" as const,
      accessRole: "user" as const,
      email: "person@example.com",
    },
    sessionCookie: "signed-cookie",
  })),
  revokeAuthSessions: mock(async () => undefined),
  setAuthSessionCookie: mock(() => undefined),
};

mock.module("$lib/server/auth", () => mockedServerAuth);

function createCookies() {
  return {
    delete: mock(() => undefined),
    get: mock(() => undefined),
    set: mock(() => undefined),
  };
}

afterEach(() => {
  mockedServerAuth.buildAnonymousAuthContext.mockClear();
  mockedServerAuth.clearAuthSessionCookies.mockClear();
  mockedServerAuth.createSessionCookieFromIdToken.mockClear();
  mockedServerAuth.revokeAuthSessions.mockClear();
  mockedServerAuth.setAuthSessionCookie.mockClear();
});

async function loadRouteModule() {
  return import(
    `../src/routes/auth/session/+server.ts?test=${Date.now()}-${Math.random()}`
  );
}

describe("/auth/session route", () => {
  it("returns the current server auth state on GET", async () => {
    const route = await loadRouteModule();

    const response = await route.GET({
      locals: {
        auth: {
          userId: "anon-123",
          authState: "anonymous",
          accessRole: "anonymous",
          email: null,
        },
      },
    });

    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
  });

  it("rejects a missing idToken payload on POST", async () => {
    const route = await loadRouteModule();

    const response = await route.POST({
      cookies: createCookies(),
      request: new Request("http://localhost/auth/session", {
        method: "POST",
        body: JSON.stringify({}),
        headers: {
          "Content-Type": "application/json",
        },
      }),
    });

    expect(response.status).toBe(400);
    expect(
      mockedServerAuth.createSessionCookieFromIdToken,
    ).not.toHaveBeenCalled();
  });

  it("creates a session cookie and returns auth payload on valid POST", async () => {
    const route = await loadRouteModule();
    const cookies = createCookies();

    const response = await route.POST({
      cookies,
      request: new Request("http://localhost/auth/session", {
        method: "POST",
        body: JSON.stringify({ idToken: "fresh-token" }),
        headers: {
          "Content-Type": "application/json",
        },
      }),
    });

    expect(response.status).toBe(200);
    expect(
      mockedServerAuth.createSessionCookieFromIdToken,
    ).toHaveBeenCalledWith("fresh-token");
    expect(mockedServerAuth.setAuthSessionCookie).toHaveBeenCalledWith(
      cookies,
      "signed-cookie",
    );
    expect(await response.json()).toEqual({
      userId: "user-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });

  it("clears the current session on DELETE", async () => {
    const route = await loadRouteModule();
    const cookies = createCookies();

    const response = await route.DELETE({
      cookies,
    });

    expect(response.status).toBe(200);
    expect(mockedServerAuth.clearAuthSessionCookies).toHaveBeenCalledWith(
      cookies,
    );
    expect(await response.json()).toEqual({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
  });
});
