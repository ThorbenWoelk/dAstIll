import { describe, expect, it, mock } from "bun:test";

import {
  canPersistServerPreferences,
  hydrateAuthenticatedPreferences,
} from "../src/lib/workspace/home-workspace-preferences";
import type { UserPreferences } from "../src/lib/types";

const samplePreferences: UserPreferences = {
  channel_order: ["channel-a"],
  channel_sort_mode: "custom",
  vocabulary_replacements: [
    {
      from: "teh",
      to: "the",
      added_at: "2026-01-01T00:00:00.000Z",
    },
  ],
};

describe("hydrateAuthenticatedPreferences", () => {
  it("skips before auth is ready so anonymous defaults are not treated as loaded", async () => {
    const getPreferences = mock(async () => samplePreferences);

    await expect(
      hydrateAuthenticatedPreferences({
        authReady: false,
        auth: {
          authState: "authenticated",
          userId: "user-1",
        },
        getPreferences,
      }),
    ).resolves.toEqual({ status: "skipped", reason: "auth-not-ready" });
    expect(getPreferences).not.toHaveBeenCalled();
  });

  it("skips anonymous sessions", async () => {
    const getPreferences = mock(async () => samplePreferences);

    await expect(
      hydrateAuthenticatedPreferences({
        authReady: true,
        auth: {
          authState: "anonymous",
          userId: "anon-1",
        },
        getPreferences,
      }),
    ).resolves.toEqual({ status: "skipped", reason: "unauthenticated" });
    expect(getPreferences).not.toHaveBeenCalled();
  });

  it("loads preferences only for authenticated ready sessions", async () => {
    const getPreferences = mock(async () => samplePreferences);

    await expect(
      hydrateAuthenticatedPreferences({
        authReady: true,
        auth: {
          authState: "authenticated",
          userId: "user-1",
        },
        getPreferences,
      }),
    ).resolves.toEqual({
      status: "loaded",
      preferences: samplePreferences,
      scopeKey: "user:user-1",
    });
    expect(getPreferences).toHaveBeenCalledTimes(1);
  });

  it("marks failed loads so callers can keep server saves disabled", async () => {
    await expect(
      hydrateAuthenticatedPreferences({
        authReady: true,
        auth: {
          authState: "authenticated",
          userId: "user-1",
        },
        getPreferences: async () => {
          throw new Error("network down");
        },
      }),
    ).resolves.toEqual({ status: "failed" });
  });
});

describe("canPersistServerPreferences", () => {
  it("blocks saves until preferences were hydrated for the active auth scope", () => {
    expect(
      canPersistServerPreferences({
        preferencesHydrated: false,
        preferencesScopeKey: null,
        auth: { authState: "authenticated", userId: "user-1" },
      }),
    ).toBe(false);

    expect(
      canPersistServerPreferences({
        preferencesHydrated: true,
        preferencesScopeKey: "user:user-1",
        auth: { authState: "authenticated", userId: "user-1" },
      }),
    ).toBe(true);
  });

  it("blocks saves when the hydrated scope does not match the current user", () => {
    expect(
      canPersistServerPreferences({
        preferencesHydrated: true,
        preferencesScopeKey: "user:user-1",
        auth: { authState: "authenticated", userId: "user-2" },
      }),
    ).toBe(false);
  });

  it("blocks saves for anonymous sessions even if hydration flags are stale", () => {
    expect(
      canPersistServerPreferences({
        preferencesHydrated: true,
        preferencesScopeKey: "user:user-1",
        auth: { authState: "anonymous", userId: "anon-1" },
      }),
    ).toBe(false);
  });
});
