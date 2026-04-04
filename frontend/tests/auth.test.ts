import { describe, expect, it } from "bun:test";

import {
  buildAnonymousAuthContext,
  buildAuthenticatedAuthContext,
  normalizeRedirectTarget,
} from "../src/lib/auth";

describe("auth helpers", () => {
  it("builds anonymous and authenticated auth contexts", () => {
    expect(buildAnonymousAuthContext()).toEqual({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    expect(
      buildAuthenticatedAuthContext("firebase-user-123", "person@example.com"),
    ).toEqual({
      userId: "firebase-user-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });

  it("normalizes redirect targets to safe in-app paths", () => {
    expect(normalizeRedirectTarget("/workspace?channel=abc")).toBe(
      "/workspace?channel=abc",
    );
    expect(normalizeRedirectTarget("https://example.com")).toBe("/");
    expect(normalizeRedirectTarget("//evil.example")).toBe("/");
    expect(normalizeRedirectTarget("")).toBe("/");
    expect(normalizeRedirectTarget(null)).toBe("/");
  });
});
