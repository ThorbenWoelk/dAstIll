import { describe, expect, it } from "bun:test";

import { HOME_WORKSPACE_HREF, signOutAndReloadHome } from "../src/lib/logout";

describe("logout helpers", () => {
  it("signs out before reloading the home workspace", async () => {
    const calls: string[] = [];
    const location = { href: "/chat" };

    await signOutAndReloadHome({
      signOut: async () => {
        calls.push("sign-out");
      },
      location,
    });

    expect(calls).toEqual(["sign-out"]);
    expect(location.href).toBe(HOME_WORKSPACE_HREF);
  });

  it("can sign out without a browser location", async () => {
    let signedOut = false;

    await signOutAndReloadHome({
      signOut: async () => {
        signedOut = true;
      },
      location: undefined,
    });

    expect(signedOut).toBe(true);
  });
});
