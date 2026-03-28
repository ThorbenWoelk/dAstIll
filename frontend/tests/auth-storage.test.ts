import { describe, expect, it } from "bun:test";

import {
  getAuthStorageScopeKey,
  getScopedStorageKey,
} from "../src/lib/auth-storage";

describe("auth storage scope", () => {
  it("namespaces authenticated users by uid", () => {
    expect(
      getAuthStorageScopeKey({
        authState: "authenticated",
        userId: "user-123",
      }),
    ).toBe("user:user-123");
  });

  it("namespaces anonymous users by uid when available", () => {
    expect(
      getAuthStorageScopeKey({
        authState: "anonymous",
        userId: "anon-123",
      }),
    ).toBe("anonymous:anon-123");
  });

  it("falls back to a bootstrap namespace before anonymous auth is ready", () => {
    expect(
      getAuthStorageScopeKey({
        authState: "anonymous",
        userId: null,
      }),
    ).toBe("anonymous:bootstrap");
  });

  it("builds stable scoped storage keys", () => {
    expect(
      getScopedStorageKey("workspace-search-session", "user:user-123"),
    ).toBe("workspace-search-session:user:user-123");
  });
});
