import { afterEach, describe, expect, it, mock } from "bun:test";

mock.module("$app/environment", () => ({
  dev: false,
}));

mock.module("$env/dynamic/private", () => ({
  env: process.env,
}));

const originalEnv = {
  BACKEND_API_BASE: process.env.BACKEND_API_BASE,
  BACKEND_IDENTITY_AUDIENCE: process.env.BACKEND_IDENTITY_AUDIENCE,
  BACKEND_PROXY_TOKEN: process.env.BACKEND_PROXY_TOKEN,
  OPERATOR_EMAIL_ALLOWLIST: process.env.OPERATOR_EMAIL_ALLOWLIST,
};

function restoreEnv() {
  for (const [key, value] of Object.entries(originalEnv)) {
    if (value === undefined) {
      delete process.env[key];
      continue;
    }

    process.env[key] = value;
  }
}

afterEach(() => {
  restoreEnv();
});

async function loadAuthModule() {
  return import(`../src/lib/server/auth?test=${Date.now()}-${Math.random()}`);
}

describe("server auth runtime config", () => {
  it("loads proxy config without requiring legacy admin password config", async () => {
    process.env.BACKEND_API_BASE = "https://backend.example.com";
    process.env.BACKEND_PROXY_TOKEN = "proxy-secret";
    process.env.BACKEND_IDENTITY_AUDIENCE = "https://backend.example.com";

    const auth = await loadAuthModule();

    expect(auth.getAuthRuntimeConfig()).toEqual({
      backendApiBase: "https://backend.example.com",
      backendProxyToken: "proxy-secret",
      backendIdentityAudience: "https://backend.example.com",
    });
  });
});

describe("firebase auth context helpers", () => {
  it("maps anonymous, user, and operator access roles", async () => {
    process.env.OPERATOR_EMAIL_ALLOWLIST =
      "operator@example.com, OWNER@example.com";

    const auth = await loadAuthModule();

    expect(auth.buildAnonymousAuthContext()).toEqual({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
    expect(
      auth.buildAuthenticatedAuthContext("uid-user", "person@example.com"),
    ).toEqual({
      userId: "uid-user",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
    expect(
      auth.buildAuthenticatedAuthContext("uid-operator", "OWNER@example.com"),
    ).toEqual({
      userId: "uid-operator",
      authState: "authenticated",
      accessRole: "operator",
      email: "OWNER@example.com",
    });
  });

  it("writes and clears the firebase session cookie with strict defaults", async () => {
    const auth = await loadAuthModule();
    const setCalls: Array<{
      name: string;
      value: string;
      options: Record<string, unknown>;
    }> = [];
    const deleteCalls: Array<{
      name: string;
      options: Record<string, unknown>;
    }> = [];

    const cookies = {
      set(name: string, value: string, options: Record<string, unknown>) {
        setCalls.push({ name, value, options });
      },
      delete(name: string, options: Record<string, unknown>) {
        deleteCalls.push({ name, options });
      },
    };

    auth.setAuthSessionCookie(cookies, "firebase-session-cookie");
    auth.clearAuthSessionCookies(cookies);

    expect(setCalls).toEqual([
      {
        name: "__session",
        value: "firebase-session-cookie",
        options: expect.objectContaining({
          path: "/",
          httpOnly: true,
          sameSite: "strict",
          maxAge: 60 * 60 * 24 * 7,
          secure: true,
        }),
      },
    ]);
    expect(deleteCalls).toEqual([
      {
        name: "__session",
        options: expect.objectContaining({
          path: "/",
          httpOnly: true,
          sameSite: "strict",
          secure: true,
        }),
      },
      {
        name: "dastill-session",
        options: expect.objectContaining({
          path: "/",
          httpOnly: true,
          sameSite: "strict",
          secure: true,
        }),
      },
    ]);
  });
});
