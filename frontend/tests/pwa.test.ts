import { describe, expect, it } from "bun:test";

import {
  canRegisterServiceWorker,
  registerAppServiceWorker,
} from "../src/lib/pwa";

describe("canRegisterServiceWorker", () => {
  it("allows registration over https", () => {
    expect(
      canRegisterServiceWorker({
        protocol: "https:",
        hostname: "dastill.example",
      }),
    ).toBe(true);
  });

  it("allows registration for local development hosts", () => {
    expect(
      canRegisterServiceWorker({
        protocol: "http:",
        hostname: "localhost",
      }),
    ).toBe(true);
    expect(
      canRegisterServiceWorker({
        protocol: "http:",
        hostname: "127.0.0.1",
      }),
    ).toBe(true);
  });

  it("rejects insecure remote hosts", () => {
    expect(
      canRegisterServiceWorker({
        protocol: "http:",
        hostname: "dastill.example",
      }),
    ).toBe(false);
  });
});

describe("registerAppServiceWorker", () => {
  it("registers the default service worker script when allowed", async () => {
    const registrations: string[] = [];

    const result = await registerAppServiceWorker(
      {
        serviceWorker: {
          register: async (scriptUrl: string) => {
            registrations.push(scriptUrl);
            return { scope: "/" };
          },
        },
      },
      {
        protocol: "https:",
        hostname: "dastill.example",
      },
    );

    expect(result).toBe(true);
    expect(registrations).toEqual(["/sw.js"]);
  });

  it("skips registration when the context is not eligible", async () => {
    const registrations: string[] = [];

    const result = await registerAppServiceWorker(
      {
        serviceWorker: {
          register: async (scriptUrl: string) => {
            registrations.push(scriptUrl);
            return { scope: "/" };
          },
        },
      },
      {
        protocol: "http:",
        hostname: "dastill.example",
      },
    );

    expect(result).toBe(false);
    expect(registrations).toEqual([]);
  });
});
