import { describe, expect, it, mock } from "bun:test";

import {
  resetMiniReaderForAuthScopeChange,
  shouldRedirectMiniToLogin,
  shouldReloadMiniForAuthScope,
} from "../src/lib/mini/mini-auth-scope";

describe("shouldReloadMiniForAuthScope", () => {
  it("waits for auth readiness", () => {
    expect(
      shouldReloadMiniForAuthScope({
        authReady: false,
        loadedAuthScopeKey: "anonymous:bootstrap",
        loadingAuthScopeKey: null,
        authScopeKey: "user:123",
      }),
    ).toBe(false);
  });

  it("reloads when the loaded scope differs from the active auth scope", () => {
    expect(
      shouldReloadMiniForAuthScope({
        authReady: true,
        loadedAuthScopeKey: "user:aaa",
        loadingAuthScopeKey: null,
        authScopeKey: "user:bbb",
      }),
    ).toBe(true);
  });

  it("reloads on the first authenticated visit", () => {
    expect(
      shouldReloadMiniForAuthScope({
        authReady: true,
        loadedAuthScopeKey: null,
        loadingAuthScopeKey: null,
        authScopeKey: "user:aaa",
      }),
    ).toBe(true);
  });

  it("does not duplicate an in-flight reload for the current auth scope", () => {
    expect(
      shouldReloadMiniForAuthScope({
        authReady: true,
        loadedAuthScopeKey: "user:aaa",
        loadingAuthScopeKey: "user:bbb",
        authScopeKey: "user:bbb",
      }),
    ).toBe(false);
  });

  it("does not reload when the loaded scope already matches", () => {
    expect(
      shouldReloadMiniForAuthScope({
        authReady: true,
        loadedAuthScopeKey: "user:aaa",
        loadingAuthScopeKey: null,
        authScopeKey: "user:aaa",
      }),
    ).toBe(false);
  });
});

describe("shouldRedirectMiniToLogin", () => {
  it("redirects anonymous and unknown auth states", () => {
    expect(shouldRedirectMiniToLogin("anonymous")).toBe(true);
    expect(shouldRedirectMiniToLogin("unauthenticated")).toBe(true);
  });

  it("keeps authenticated sessions on /mini", () => {
    expect(shouldRedirectMiniToLogin("authenticated")).toBe(false);
  });
});

describe("resetMiniReaderForAuthScopeChange", () => {
  it("clears reader, preferences, highlights, and vocabulary before the next scope loads", () => {
    const calls: string[] = [];
    const target = {
      clearReaderState: mock(() => {
        calls.push("clearReaderState");
      }),
      clearPreferences: mock(() => {
        calls.push("clearPreferences");
      }),
      resetHighlights: mock(() => {
        calls.push("resetHighlights");
      }),
      resetVocabulary: mock(() => {
        calls.push("resetVocabulary");
      }),
    };

    resetMiniReaderForAuthScopeChange(target);

    expect(calls).toEqual([
      "clearReaderState",
      "clearPreferences",
      "resetHighlights",
      "resetVocabulary",
    ]);
  });
});
