import { afterEach, beforeEach, describe, expect, it } from "bun:test";

import {
  resolveBrowserAuthOrigin,
  resolveSystemBrowserLoginUrl,
} from "../src/lib/browser-auth";

const originalWindow = globalThis.window;
const originalNavigator = globalThis.navigator;

beforeEach(() => {
  Object.defineProperty(globalThis, "window", {
    value: {
      __TAURI_INTERNALS__: {},
      location: {
        href: "http://localhost:3543/login",
      },
      open: () => null,
    },
    configurable: true,
  });
  Object.defineProperty(globalThis, "navigator", {
    value: {
      userAgent: "Android sdk_gphone64_arm64",
    },
    configurable: true,
  });
});

afterEach(() => {
  if (originalWindow === undefined) {
    delete (globalThis as typeof globalThis & { window?: unknown }).window;
  } else {
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
    });
  }

  if (originalNavigator === undefined) {
    delete (globalThis as typeof globalThis & { navigator?: unknown })
      .navigator;
  } else {
    Object.defineProperty(globalThis, "navigator", {
      value: originalNavigator,
      configurable: true,
    });
  }
});

describe("browser auth helpers", () => {
  it("keeps the current origin when no explicit browser auth base URL is configured", () => {
    expect(
      resolveBrowserAuthOrigin(new URL("http://localhost:3543/login")),
    ).toBe("http://localhost:3543");
  });

  it("builds the system-browser login URL with a safe redirect target", () => {
    expect(resolveSystemBrowserLoginUrl("/chat?conversation=123")).toBe(
      "http://localhost:3543/login?redirectTo=%2Fchat%3Fconversation%3D123&mobileBrowserAuth=1",
    );
  });
});
