import { afterEach, describe, expect, it, mock } from "bun:test";

mock.module("$app/environment", () => ({
  dev: false,
}));

mock.module("$env/dynamic/private", () => ({
  env: process.env,
}));

const originalEnv = {
  ADMIN_PASSWORD: process.env.ADMIN_PASSWORD,
  BACKEND_API_BASE: process.env.BACKEND_API_BASE,
  BACKEND_IDENTITY_AUDIENCE: process.env.BACKEND_IDENTITY_AUDIENCE,
  BACKEND_PROXY_TOKEN: process.env.BACKEND_PROXY_TOKEN,
};

function restoreEnv() {
  for (const [key, value] of Object.entries(originalEnv)) {
    if (value === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = value;
    }
  }
}

afterEach(() => {
  restoreEnv();
});

async function loadAuthModule() {
  return import("../src/lib/server/auth");
}

describe("server auth runtime config", () => {
  it("loads proxy config without requiring ADMIN_PASSWORD", async () => {
    process.env.BACKEND_API_BASE = "https://backend.example.com";
    process.env.BACKEND_PROXY_TOKEN = "proxy-secret";
    process.env.BACKEND_IDENTITY_AUDIENCE = "https://backend.example.com";
    delete process.env.ADMIN_PASSWORD;

    const auth = await loadAuthModule();

    expect(auth.getAuthRuntimeConfig()).toEqual({
      backendApiBase: "https://backend.example.com",
      backendProxyToken: "proxy-secret",
      backendIdentityAudience: "https://backend.example.com",
    });
  });

  it("treats admin auth as unavailable when ADMIN_PASSWORD is unset", async () => {
    delete process.env.ADMIN_PASSWORD;

    const auth = await loadAuthModule();

    expect(auth.isValidAdminPassword("secret")).toBeFalse();
    expect(auth.readAdminSession("invalid.token.parts")).toBeNull();
  });

  it("creates readable admin sessions only when ADMIN_PASSWORD is configured", async () => {
    process.env.ADMIN_PASSWORD = "secret";

    const auth = await loadAuthModule();

    const token = auth.createAdminSessionToken();
    const session = auth.readAdminSession(token);

    expect(auth.isValidAdminPassword("secret")).toBeTrue();
    expect(auth.isValidAdminPassword("nope")).toBeFalse();
    expect(session).not.toBeNull();
    expect(session?.sid).toHaveLength(32);
  });

  it("invalidates existing admin sessions if ADMIN_PASSWORD is later removed", async () => {
    process.env.ADMIN_PASSWORD = "secret";
    const auth = await loadAuthModule();
    const token = auth.createAdminSessionToken();

    delete process.env.ADMIN_PASSWORD;

    expect(auth.readAdminSession(token)).toBeNull();
  });
});
