import { expect, test } from "@playwright/test";

import { PLAYWRIGHT_MAINTENANCE_MODE } from "./runtime-mode";
import { openFreshGuestPage } from "./test-helpers";

test.skip(
  !PLAYWRIGHT_MAINTENANCE_MODE,
  "Maintenance mobile-home assertions only apply when the repo runtime mode is maintenance.",
);

test.use({
  viewport: { width: 390, height: 844 },
  isMobile: true,
  hasTouch: true,
});

test("home route keeps the maintenance CTA visible on mobile", async ({
  page,
}) => {
  await openFreshGuestPage(page, "/");

  await expect(
    page.getByRole("link", {
      name: "Sign in and continue to dastill-mini",
    }),
  ).toBeVisible();
  await expect(page.locator("#workspace")).toHaveCount(0);
});
