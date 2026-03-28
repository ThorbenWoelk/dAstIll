import { describe, expect, it } from "bun:test";
import { get } from "svelte/store";

import { AuthRequiredError } from "../src/lib/api-client";
import {
  authRequiredNotice,
  dismissAuthRequiredNotice,
  presentAuthRequiredNotice,
  presentAuthRequiredNoticeIfNeeded,
  setFeatureGuideSuppressesAuthRequiredNotice,
} from "../src/lib/auth-required-notice";

describe("auth-required-notice", () => {
  it("does not open the notice while the feature guide suppresses it", () => {
    dismissAuthRequiredNotice();
    setFeatureGuideSuppressesAuthRequiredNotice(true);
    presentAuthRequiredNotice("Sign in to continue.");
    expect(get(authRequiredNotice)).toBe(null);
    setFeatureGuideSuppressesAuthRequiredNotice(false);
    presentAuthRequiredNotice("After guide");
    expect(get(authRequiredNotice)).toBe("After guide");
    dismissAuthRequiredNotice();
  });

  it("presentAuthRequiredNoticeIfNeeded opens the modal for AuthRequiredError", () => {
    dismissAuthRequiredNotice();
    expect(presentAuthRequiredNoticeIfNeeded(new AuthRequiredError())).toBe(
      true,
    );
    expect(get(authRequiredNotice)).toBe("Sign in to continue.");
    dismissAuthRequiredNotice();
  });

  it("presentAuthRequiredNoticeIfNeeded returns false for unrelated errors", () => {
    dismissAuthRequiredNotice();
    expect(presentAuthRequiredNoticeIfNeeded(new Error("Network error"))).toBe(
      false,
    );
    expect(get(authRequiredNotice)).toBe(null);
  });
});
